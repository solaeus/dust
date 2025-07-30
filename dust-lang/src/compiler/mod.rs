//! The Dust compiler and its accessories.
//!
//! This module provides two compilation options:
//! - [`compile`] is a simple function that borrows a string and returns a chunk, handling
//!   compilation and turning any resulting error into a [`DustError`], which can easily display a
//!   detailed report.
//! - [`Compiler`] is created with a [`Lexer`] and potentially emits a [`CompileError`] if the
//!   input is invalid.
//!
//! # Errors
//!
//! The compiler can return errors due to:
//!     - Lexing errors
//!     - Parsing errors
//!     - Type conflicts
//!
//! It is a logic error to call [`Compiler::finish`] on a compiler that has emitted an error and
//! pass that chunk to the VM.
#![macro_use]

mod compile_error;
mod compile_mode;
mod global;
mod item;
mod local;
mod module;
mod parse_rule;
mod path;
mod standard_library;
mod type_checks;

pub use compile_error::CompileError;
use compile_mode::CompileMode;
pub use global::Global;
use indexmap::IndexMap;
pub use item::Item;
pub use local::{BlockScope, Local};
pub use module::Module;
use parse_rule::{ParseRule, Precedence};
pub use path::Path;
use tracing::{Level, debug, info, span, trace};
use type_checks::{check_math_type, check_math_types};

use std::{cell::RefCell, collections::HashSet, mem::replace, rc::Rc, sync::Arc};

use crate::{
    Address, Chunk, DustError, FunctionType, Instruction, Lexer, List, NativeFunction, Operation,
    Program, Span, Token, TokenKind, Type, Value,
    compiler::standard_library::apply_standard_library,
    instruction::{Jump, Load, MemoryKind, OperandType},
    r#type::ConcreteType,
};

/// Compiles the input and returns a chunk.
///
/// # Example
///
/// ```
/// # use dust_lang::{compile, Chunk, FullChunk};
/// let source = "40 + 2 == 42";
/// let chunk = compile::<FullChunk>(source).unwrap();
///
/// assert_eq!(chunk.instructions().len(), 6);
/// ```
pub fn compile(source: &'_ str) -> Result<Program, DustError<'_>> {
    let compiler = Compiler::new();

    compiler
        .compile_program(None, source)
        .map_err(|error| DustError::compile(error, source))
}

/// The Dust compiler assembles a [`Chunk`] for the Dust VM. Any unrecognized symbols, disallowed
/// syntax or conflicting type usage will result in an error.
///
/// See the [`compile`] function an example of how to create and use a Compiler.
#[derive(Debug)]
pub struct Compiler {
    allow_native_functions: bool,
}

impl Compiler {
    pub fn new() -> Self {
        Self {
            allow_native_functions: false,
        }
    }

    pub fn compile_library(
        &mut self,
        library_name: &str,
        source: &str,
    ) -> Result<Module, CompileError> {
        let logging = span!(Level::INFO, "Compile Library");
        let _enter = logging.enter();

        let name = Path::new(library_name).ok_or_else(|| CompileError::InvalidLibraryPath {
            found: library_name.to_string(),
        })?;
        let lexer = Lexer::new(source);
        let mode = CompileMode::Module {
            name,
            module: Module::new(),
        };
        let item_scope = HashSet::new();
        let main_module = Rc::new(RefCell::new(Module::new()));
        let globals = Rc::new(RefCell::new(IndexMap::new()));
        let prototypes = Rc::new(RefCell::new(Vec::new()));

        if !self.allow_native_functions {
            apply_standard_library(&mut main_module.borrow_mut());
        }

        let mut chunk_compiler = ChunkCompiler::new(
            lexer,
            mode,
            item_scope,
            main_module.clone(),
            globals,
            prototypes,
        )?;
        chunk_compiler.allow_native_functions = self.allow_native_functions;

        chunk_compiler.compile()?;

        drop(chunk_compiler);
        debug_assert!(Rc::strong_count(&main_module) == 1);

        let main_module = Rc::unwrap_or_clone(main_module).into_inner();

        Ok(main_module)
    }

    pub fn compile_program(
        &self,
        program_name: Option<&str>,
        source: &str,
    ) -> Result<Program, CompileError> {
        let logging = span!(Level::INFO, "Compile Program");
        let _enter = logging.enter();

        let program_name = program_name.unwrap_or("main");
        let program_path =
            Path::new(program_name).ok_or_else(|| CompileError::InvalidProgramPath {
                found: program_name.to_string(),
            })?;
        let lexer = Lexer::new(source);
        let mode = CompileMode::Main;
        let item_scope = HashSet::new();
        let main_module = Rc::new(RefCell::new(Module::new()));
        let globals = Rc::new(RefCell::new(IndexMap::new()));
        let prototypes = Rc::new(RefCell::new(Vec::new()));

        if !self.allow_native_functions {
            apply_standard_library(&mut main_module.borrow_mut());
        }

        let mut chunk_compiler =
            ChunkCompiler::new(lexer, mode, item_scope, main_module, globals, prototypes)?;
        chunk_compiler.allow_native_functions = true;

        chunk_compiler.compile()?;

        let cell_count = chunk_compiler.globals.borrow().len() as u16;
        let register_count = chunk_compiler.next_register_index_without_reclaiming();
        let instructions = chunk_compiler
            .instructions
            .into_iter()
            .map(|(instruction, _, _)| instruction)
            .collect();
        let locals = chunk_compiler.locals.into_iter().collect();
        let main_chunk = Chunk {
            name: Some(program_path),
            r#type: chunk_compiler.r#type,
            instructions,
            constants: chunk_compiler.constants,
            locals,
            register_count,
            prototype_index: 0,
        };
        let prototypes = Rc::into_inner(chunk_compiler.prototypes)
            .expect("Unnecessary clone of prototypes")
            .into_inner();
        let prototypes = Arc::new(prototypes);

        Ok(Program {
            main_chunk,
            cell_count,
            prototypes,
        })
    }
}

impl Default for Compiler {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug)]
pub struct Arguments {
    pub values: Vec<(Address, OperandType)>,
    pub types: Vec<Type>,
}

#[derive(Debug)]
pub(crate) struct ChunkCompiler<'a> {
    /// Indication of what the compiler will produce when it finishes. This value should never be
    /// mutated.
    mode: CompileMode,

    /// Used to get tokens for the compiler.
    lexer: Lexer<'a>,

    /// Type of the function being compiled. This is assigned to the chunk when [`Compiler::finish`]
    /// is called.
    r#type: FunctionType,

    /// Instructions, along with their types and positions, that have been compiled. The
    /// instructions and positions are assigned to the chunk when [`Compiler::finish`] is called.
    /// The types are discarded after compilation.
    instructions: Vec<(Instruction, Type, Span)>,

    /// Constant values that have been compiled, including function prototypes. These are assigned to
    /// the chunk when [`Compiler::finish`] is called.
    constants: Vec<Value>,

    /// Block-local variables.
    locals: IndexMap<Path, Local>,

    minimum_register_index: usize,

    reclaimable_register_indexes: Vec<usize>,

    /// Index of the current block. This is used to determine the scope of locals and is incremented
    /// when a new block is entered.
    block_index: u8,

    /// This is mutated during compilation to match the current block and is used to test if a local
    /// variable is in scope.
    current_block_scope: BlockScope,

    /// This is mutated during compilation as items are brought into scope by `use` statements or
    /// are invoked by their full path. It is used to test if an item is in scope.
    current_item_scope: HashSet<Path>,

    /// Index of the Chunk in its parent's prototype list. This is set to 0 for the main chunk but
    /// that value is never read because the main chunk is not a callable function.
    prototype_index: usize,

    main_module: Rc<RefCell<Module>>,

    globals: Rc<RefCell<IndexMap<Path, Global>>>,

    prototypes: Rc<RefCell<Vec<Arc<Chunk>>>>,

    previous_statement_end: usize,
    previous_expression_end: usize,

    current_token: Token<'a>,
    current_position: Span,
    previous_token: Token<'a>,
    previous_position: Span,

    allow_native_functions: bool,
}

impl<'a> ChunkCompiler<'a> {
    fn new(
        mut lexer: Lexer<'a>,
        mode: CompileMode,
        item_scope: HashSet<Path>,
        main_module: Rc<RefCell<Module>>,
        globals: Rc<RefCell<IndexMap<Path, Global>>>,
        prototypes: Rc<RefCell<Vec<Arc<Chunk>>>>,
    ) -> Result<Self, CompileError> {
        let (current_token, current_position) = lexer.next_token()?;

        Ok(ChunkCompiler {
            mode,
            r#type: FunctionType::default(),
            instructions: Vec::new(),
            constants: Vec::new(),
            locals: IndexMap::new(),
            lexer,
            minimum_register_index: 0,
            reclaimable_register_indexes: Vec::new(),
            block_index: 0,
            current_block_scope: BlockScope::default(),
            current_item_scope: item_scope,
            prototype_index: 0,
            main_module,
            globals,
            prototypes,
            current_token,
            current_position,
            previous_token: Token::Eof,
            previous_position: Span(0, 0),
            previous_statement_end: 0,
            previous_expression_end: 0,
            allow_native_functions: false,
        })
    }

    /// Compiles the source (which is in the lexer) while checking for errors and returning a
    /// [`CompileError`] if any are found. After calling this function, check its return value for
    /// an error, then call [`Compiler::finish`] to get the compiled chunk.
    pub fn compile(&mut self) -> Result<(), CompileError> {
        for (top_level_path, (_, _)) in self.main_module.borrow().items.iter() {
            self.current_item_scope.insert(top_level_path.clone());
        }

        let logging = span!(Level::INFO, "Compile");
        let _enter = logging.enter();

        info!(
            "Begin chunk with `{}` at {}",
            self.current_token, self.current_position
        );

        while !matches!(self.current_token, Token::Eof | Token::RightBrace) {
            self.parse(Precedence::None)?;
        }

        self.parse_implicit_return()?;

        if self.instructions.is_empty() {
            let r#return =
                Instruction::r#return(false as usize, Address::default(), OperandType::NONE);

            self.emit_instruction(r#return, Type::None, self.current_position);
        }

        info!("End chunk");

        Ok(())
    }

    fn is_eof(&self) -> bool {
        matches!(self.current_token, Token::Eof)
    }

    fn next_register_index(&mut self) -> usize {
        if let Some(index) = self.reclaimable_register_indexes.pop() {
            trace!("Reclaiming boolean memory at index {index}");

            return index;
        }

        self.instructions
            .iter()
            .fold(self.minimum_register_index, |acc, (instruction, _, _)| {
                if instruction.yields_value()
                    && instruction.a_field() >= acc
                    && instruction.a_memory_kind() == MemoryKind::REGISTER
                {
                    instruction.a_field() + 1
                } else {
                    acc
                }
            })
    }

    fn next_register_index_without_reclaiming(&mut self) -> usize {
        self.instructions
            .iter()
            .fold(self.minimum_register_index, |acc, (instruction, _, _)| {
                if instruction.yields_value()
                    && instruction.a_field() >= acc
                    && instruction.a_memory_kind() == MemoryKind::REGISTER
                {
                    instruction.a_field() + 1
                } else {
                    acc
                }
            })
    }

    /// Advances to the next token emitted by the lexer.
    fn advance(&mut self) -> Result<(), CompileError> {
        if self.is_eof() {
            return Ok(());
        }

        let (new_token, position) = self.lexer.next_token()?;

        trace!(
            "Parsing {} at {}",
            new_token.to_string(),
            position.to_string()
        );

        self.previous_token = replace(&mut self.current_token, new_token);
        self.previous_position = replace(&mut self.current_position, position);

        Ok(())
    }

    /// Returns the local with the given identifier.
    fn get_local(&self, path: &Path) -> Option<&Local> {
        self.locals.get_key_value(path).map(|(_, local)| local)
    }

    /// Adds a new local to `self.locals` and returns a tuple holding the index of the new local and
    /// the index of its identifier in `self.string_constants`.
    fn declare_local(
        &mut self,
        identifier: Path,
        address: Address,
        r#type: Type,
        is_mutable: bool,
        scope: BlockScope,
    ) {
        info!("Declaring local {identifier} at {address}");

        if address.memory == MemoryKind::REGISTER {
            self.minimum_register_index += 1;
        }

        self.locals
            .insert(identifier, Local::new(address, r#type, is_mutable, scope));
    }

    fn declare_global(&mut self, identifier: Path, r#type: Type, is_mutable: bool) -> usize {
        info!("Declaring global {identifier}");

        let mut globals = self.globals.borrow_mut();
        let cell_index = globals.len();

        globals.insert(identifier, Global::new(r#type, is_mutable));

        cell_index
    }

    fn clone_item(&self, item_path: &Path) -> Result<(Item, Span), CompileError> {
        if let CompileMode::Module { module, .. } = &self.mode {
            if let Some(found) = module.find_item(item_path) {
                return Ok(found.clone());
            }
        } else if let Some(found) = self.main_module.borrow().find_item(item_path) {
            return Ok(found.clone());
        }

        Err(CompileError::UnknownItem {
            item_name: item_path.inner().to_string(),
            position: self.previous_position,
        })
    }

    fn push_constant_or_get_index(&mut self, constant: Value) -> usize {
        let found_index = self
            .constants
            .iter()
            .position(|existing| existing == &constant);

        if let Some(index) = found_index {
            index
        } else {
            let index = self.constants.len();

            self.constants.push(constant);

            index
        }
    }

    fn allow(&mut self, allowed: Token) -> Result<bool, CompileError> {
        if self.current_token == allowed {
            self.advance()?;

            Ok(true)
        } else {
            Ok(false)
        }
    }

    fn expect(&mut self, expected: Token) -> Result<(), CompileError> {
        if self.current_token == expected {
            self.advance()
        } else {
            Err(CompileError::ExpectedToken {
                expected: expected.kind(),
                found: self.current_token.to_owned(),
                position: self.current_position,
            })
        }
    }

    fn get_last_operation(&self) -> Option<Operation> {
        self.instructions
            .last()
            .map(|(instruction, _, _)| instruction.operation())
    }

    fn get_last_operations<const COUNT: usize>(&self) -> Option<[Operation; COUNT]> {
        let mut n_operations = [Operation::RETURN; COUNT];

        for (nth, operation) in n_operations.iter_mut().rev().zip(
            self.instructions
                .iter()
                .rev()
                .map(|(instruction, _, _)| instruction.operation()),
        ) {
            *nth = operation;
        }

        Some(n_operations)
    }

    fn get_last_instruction_type(&self) -> Type {
        self.instructions
            .last()
            .map(|(_, r#type, _)| r#type.clone())
            .unwrap_or(Type::None)
    }

    /// Updates [`Self::type`] with the given [Type] as `return_type`.
    ///
    /// If [`Self::type`] is already set, it will check if the given [Type] is compatible.
    fn update_return_type(&mut self, new_return_type: Type) -> Result<(), CompileError> {
        if self.r#type.return_type != Type::None {
            self.r#type
                .return_type
                .check(&new_return_type)
                .map_err(|conflict| CompileError::ReturnTypeConflict {
                    conflict,
                    position: self.previous_position,
                })?;
        }

        self.r#type.return_type = new_return_type;

        Ok(())
    }

    fn emit_instruction(&mut self, instruction: Instruction, r#type: Type, position: Span) {
        debug!(
            "Emitting {} at {}",
            instruction.operation().to_string(),
            position.to_string()
        );

        self.instructions.push((instruction, r#type, position));
    }

    fn parse_boolean(&mut self) -> Result<(), CompileError> {
        let position = self.current_position;

        if let Token::Boolean(text) = self.current_token {
            self.advance()?;

            let boolean = self.parse_boolean_value(text);
            let destination = Address::register(self.next_register_index());
            let operand = Address::encoded(boolean as usize);
            let load =
                Instruction::load(destination, operand, OperandType::BOOLEAN, false as usize);

            self.emit_instruction(load, Type::Boolean, position);

            Ok(())
        } else {
            Err(CompileError::ExpectedToken {
                expected: TokenKind::Boolean,
                found: self.current_token.to_owned(),
                position,
            })
        }
    }

    fn parse_boolean_value(&mut self, text: &str) -> bool {
        text.parse::<bool>().unwrap() // The lexer guarantees that the text is either "true" or "false"
    }

    fn parse_byte(&mut self) -> Result<(), CompileError> {
        let position = self.current_position;

        if let Token::Byte(text) = self.current_token {
            self.advance()?;

            let byte = self.parse_byte_value(text)?;
            let destination = Address::register(self.next_register_index());
            let operand = Address::encoded(byte as usize);
            let load_encoded =
                Instruction::load(destination, operand, OperandType::BYTE, false as usize);

            self.emit_instruction(load_encoded, Type::Byte, position);

            Ok(())
        } else {
            Err(CompileError::ExpectedToken {
                expected: TokenKind::Byte,
                found: self.current_token.to_owned(),
                position,
            })
        }
    }

    fn parse_byte_value(&mut self, text: &str) -> Result<u8, CompileError> {
        u8::from_str_radix(&text[2..], 16).map_err(|error| CompileError::ParseIntError {
            error,
            position: self.previous_position,
        })
    }

    fn parse_character(&mut self) -> Result<(), CompileError> {
        let position = self.current_position;

        if let Token::Character(character) = self.current_token {
            self.advance()?;

            let destination = Address::register(self.next_register_index());
            let constant_index = self.push_constant_or_get_index(Value::Character(character));
            let operand = Address::constant(constant_index);
            let load_encoded =
                Instruction::load(destination, operand, OperandType::CHARACTER, false as usize);

            self.emit_instruction(load_encoded, Type::Character, position);

            Ok(())
        } else {
            Err(CompileError::ExpectedToken {
                expected: TokenKind::Character,
                found: self.current_token.to_owned(),
                position,
            })
        }
    }

    fn parse_float(&mut self) -> Result<(), CompileError> {
        let position = self.current_position;

        if let Token::Float(text) = self.current_token {
            self.advance()?;

            let float = Value::float(self.parse_float_value(text)?);
            let destination = Address::register(self.next_register_index());
            let constant_index = self.push_constant_or_get_index(float);
            let load_constant = Instruction::load(
                destination,
                Address::constant(constant_index),
                OperandType::FLOAT,
                false as usize,
            );

            self.emit_instruction(load_constant, Type::Float, position);

            Ok(())
        } else {
            Err(CompileError::ExpectedToken {
                expected: TokenKind::Float,
                found: self.current_token.to_owned(),
                position,
            })
        }
    }

    fn parse_float_value(&mut self, text: &str) -> Result<f64, CompileError> {
        text.parse::<f64>()
            .map_err(|error| CompileError::ParseFloatError {
                error,
                position: self.previous_position,
            })
    }

    fn parse_integer(&mut self) -> Result<(), CompileError> {
        let position = self.current_position;

        if let Token::Integer(text) = self.current_token {
            self.advance()?;

            let integer = Value::integer(self.parse_integer_value(text));
            let constant_index = self.push_constant_or_get_index(integer);
            let destination = Address::register(self.next_register_index());
            let load_constant = Instruction::load(
                destination,
                Address::constant(constant_index),
                OperandType::INTEGER,
                false as usize,
            );

            self.emit_instruction(load_constant, Type::Integer, position);

            Ok(())
        } else {
            Err(CompileError::ExpectedToken {
                expected: TokenKind::Integer,
                found: self.current_token.to_owned(),
                position,
            })
        }
    }

    fn parse_integer_value(&mut self, text: &str) -> i64 {
        let mut integer = 0_i64;
        let mut chars = text.chars().peekable();

        let is_positive = if chars.peek() == Some(&'-') {
            self.advance().unwrap();

            false
        } else {
            true
        };

        for digit in chars {
            let digit = if let Some(digit) = digit.to_digit(10) {
                digit as i64
            } else {
                continue;
            };

            integer = integer * 10 + digit;
        }

        if is_positive { integer } else { -integer }
    }

    fn parse_string(&mut self) -> Result<(), CompileError> {
        let position = self.current_position;

        if let Token::String(text) = self.current_token {
            self.advance()?;

            let string = Value::string(String::from(text));
            let constant_index = self.push_constant_or_get_index(string);
            let destination = Address::register(self.next_register_index());
            let load_constant = Instruction::load(
                destination,
                Address::constant(constant_index),
                OperandType::STRING,
                false as usize,
            );

            self.emit_instruction(load_constant, Type::String, position);

            Ok(())
        } else {
            Err(CompileError::ExpectedToken {
                expected: TokenKind::String,
                found: self.current_token.to_owned(),
                position,
            })
        }
    }

    fn parse_grouped(&mut self) -> Result<(), CompileError> {
        self.allow(Token::LeftParenthesis)?;
        self.parse_expression()?;
        self.expect(Token::RightParenthesis)?;

        Ok(())
    }

    fn parse_unary(&mut self) -> Result<(), CompileError> {
        let operator = self.current_token;
        let operator_position = self.current_position;

        self.advance()?;
        self.parse_expression()?;

        let (previous_instruction, previous_type, previous_position) =
            self.instructions.pop().unwrap();
        let (address, push_back) = self.handle_binary_argument(previous_instruction);

        if push_back {
            self.instructions.push((
                previous_instruction,
                previous_type.clone(),
                previous_position,
            ))
        }

        let destination = Address::register(self.next_register_index());
        let instruction = match operator {
            Token::Bang => Instruction::negate(destination, address, OperandType::BOOLEAN),
            Token::Minus => {
                let operand_type = match previous_type {
                    Type::Byte => OperandType::BYTE,
                    Type::Float => OperandType::FLOAT,
                    Type::Integer => OperandType::INTEGER,
                    _ => todo!("Emit type error"),
                };

                Instruction::negate(destination, address, operand_type)
            }
            _ => unreachable!(),
        };

        self.emit_instruction(instruction, previous_type, operator_position);

        Ok(())
    }

    /// Takes an instruction and returns an [`address`] that corresponds to its address and a
    /// boolean indicating whether the instruction should be pushed back onto the instruction list.
    /// If `false`, the address makes the instruction irrelevant.
    fn handle_binary_argument(&mut self, instruction: Instruction) -> (Address, bool) {
        let (address, push_back) = match instruction.operation() {
            Operation::LOAD => {
                let Load { operand, .. } = Load::from(instruction);

                (operand, false)
            }
            Operation::CALL
            | Operation::CALL_NATIVE
            | Operation::ADD
            | Operation::SUBTRACT
            | Operation::MULTIPLY
            | Operation::DIVIDE
            | Operation::MODULO
            | Operation::NEGATE => (instruction.destination(), true),
            _ => (instruction.destination(), !instruction.yields_value()),
        };

        (address, push_back)
    }

    fn parse_math_binary(&mut self) -> Result<(), CompileError> {
        let (left_instruction, left_type, left_position) =
            self.instructions
                .pop()
                .ok_or_else(|| CompileError::ExpectedExpression {
                    found: self.previous_token.to_owned(),
                    position: self.previous_position,
                })?;
        let (left, push_back_left) = self.handle_binary_argument(left_instruction);

        let left_is_mutable_variable = match left_instruction.operation() {
            Operation::LOAD => {
                let Load { operand, .. } = Load::from(left_instruction);

                if operand.memory == MemoryKind::REGISTER {
                    self.locals
                        .iter()
                        .find_map(|(_, local)| {
                            if local.address == operand {
                                Some(local.is_mutable)
                            } else {
                                None
                            }
                        })
                        .unwrap_or(false)
                } else if operand.memory == MemoryKind::CELL {
                    self.globals
                        .borrow()
                        .iter()
                        .enumerate()
                        .find_map(|(i, (_, global))| {
                            if i == operand.index {
                                Some(global.is_mutable)
                            } else {
                                None
                            }
                        })
                        .unwrap_or(false)
                } else {
                    false
                }
            }
            _ => false,
        };

        if push_back_left {
            self.instructions
                .push((left_instruction, left_type.clone(), left_position));
        }

        let operator = self.current_token;
        let operator_position = self.current_position;
        let rule = ParseRule::from(operator);
        let is_assignment = matches!(
            operator,
            Token::PlusEqual
                | Token::MinusEqual
                | Token::StarEqual
                | Token::SlashEqual
                | Token::PercentEqual
        );

        check_math_type(&left_type, operator, &left_position)?;

        if is_assignment && !left_is_mutable_variable {
            return Err(CompileError::ExpectedMutableVariable {
                found: self.previous_token.to_owned(),
                position: left_position,
            });
        }

        self.advance()?;
        self.parse_sub_expression(&rule.precedence)?;

        if is_assignment {
            self.allow(Token::Semicolon)?;
        }

        let (right_instruction, right_type, right_position) = self.instructions.pop().unwrap();
        let (right, push_back_right) = self.handle_binary_argument(right_instruction);
        let right_is_constant_zero = right_instruction.operation() == Operation::LOAD
            && right.memory == MemoryKind::CONSTANT
            && {
                match right_type {
                    Type::Byte => right.index == 0,
                    Type::Float => self.constants.get(right.index) == Some(&Value::Float(0.0)),
                    Type::Integer => self.constants.get(right.index) == Some(&Value::Integer(0)),
                    _ => false,
                }
            };

        if right_is_constant_zero && operator == Token::Slash {
            return Err(CompileError::DivisionByZero {
                position: right_position,
            });
        }

        check_math_type(&right_type, operator, &right_position)?;
        check_math_types(
            &left_type,
            &left_position,
            operator,
            &right_type,
            &right_position,
        )?;
        let right_type_kind = right_type.as_concrete_type();

        if push_back_right {
            self.instructions
                .push((right_instruction, right_type, right_position));
        }

        let r#type = if is_assignment {
            Type::None
        } else if left_type == Type::Character {
            Type::String
        } else {
            left_type.clone()
        };
        let destination = if is_assignment {
            left
        } else {
            Address::register(self.next_register_index())
        };
        let operand_type = match left_type {
            Type::Byte => OperandType::BYTE,
            Type::Character => match right_type_kind {
                ConcreteType::Character => OperandType::CHARACTER,
                ConcreteType::String => OperandType::CHARACTER_STRING,
                _ => unreachable!(),
            },
            Type::Float => OperandType::FLOAT,
            Type::Integer => OperandType::INTEGER,
            Type::String => match right_type_kind {
                ConcreteType::Character => OperandType::STRING_CHARACTER,
                ConcreteType::String => OperandType::STRING,
                _ => unreachable!(),
            },
            _ => unreachable!(),
        };
        let instruction = match operator {
            Token::Plus | Token::PlusEqual => {
                Instruction::add(destination, left, right, operand_type)
            }
            Token::Minus | Token::MinusEqual => {
                Instruction::subtract(destination, left, right, operand_type)
            }
            Token::Star | Token::StarEqual => {
                Instruction::multiply(destination, left, right, operand_type)
            }
            Token::Slash | Token::SlashEqual => {
                Instruction::divide(destination, left, right, operand_type)
            }
            Token::Percent | Token::PercentEqual => {
                Instruction::modulo(destination, left, right, operand_type)
            }
            _ => {
                return Err(CompileError::ExpectedTokenMultiple {
                    expected: &[
                        TokenKind::Plus,
                        TokenKind::PlusEqual,
                        TokenKind::Minus,
                        TokenKind::MinusEqual,
                        TokenKind::Star,
                        TokenKind::StarEqual,
                        TokenKind::Slash,
                        TokenKind::SlashEqual,
                        TokenKind::Percent,
                        TokenKind::PercentEqual,
                    ],
                    found: operator.to_owned(),
                    position: operator_position,
                });
            }
        };
        let position = Span(left_position.0, right_position.1);

        self.emit_instruction(instruction, r#type, position);

        Ok(())
    }

    fn parse_comparison_binary(&mut self) -> Result<(), CompileError> {
        if let Some(
            [
                Operation::EQUAL | Operation::LESS | Operation::LESS_EQUAL,
                _,
                _,
            ],
        ) = self.get_last_operations()
            && matches!(
                self.previous_token,
                Token::DoubleEqual
                    | Token::BangEqual
                    | Token::Greater
                    | Token::GreaterEqual
                    | Token::Less
                    | Token::LessEqual
            )
        {
            return Err(CompileError::ComparisonChain {
                position: self.current_position,
            });
        }

        let (left_instruction, left_type, left_position) =
            self.instructions
                .pop()
                .ok_or_else(|| CompileError::ExpectedExpression {
                    found: self.previous_token.to_owned(),
                    position: self.previous_position,
                })?;

        let (left, push_back_left) = self.handle_binary_argument(left_instruction);

        if push_back_left {
            self.instructions
                .push((left_instruction, left_type.clone(), left_position));
        }

        let operator = self.current_token;
        let operator_position = self.current_position;
        let rule = ParseRule::from(operator);

        self.advance()?;
        self.parse_sub_expression(&rule.precedence)?;

        let (right_instruction, right_type, right_position) =
            self.instructions
                .pop()
                .ok_or_else(|| CompileError::ExpectedExpression {
                    found: self.previous_token.to_owned(),
                    position: self.previous_position,
                })?;

        let (right, push_back_right) = self.handle_binary_argument(right_instruction);

        if left_type != right_type {
            return Err(CompileError::ComparisonTypeConflict {
                left_type,
                left_position,
                right_type,
                right_position,
            });
        }

        if push_back_right {
            self.instructions
                .push((right_instruction, right_type, right_position));
        }

        let operand_type = left_type.as_operand_type();
        let comparison = match operator {
            Token::DoubleEqual => Instruction::equal(true as usize, left, right, operand_type),
            Token::BangEqual => Instruction::equal(false as usize, left, right, operand_type),
            Token::Less => Instruction::less(true as usize, left, right, operand_type),
            Token::LessEqual => Instruction::less_equal(true as usize, left, right, operand_type),
            Token::Greater => Instruction::less_equal(false as usize, left, right, operand_type),
            Token::GreaterEqual => Instruction::less(false as usize, left, right, operand_type),
            _ => {
                return Err(CompileError::ExpectedTokenMultiple {
                    expected: &[
                        TokenKind::DoubleEqual,
                        TokenKind::BangEqual,
                        TokenKind::Less,
                        TokenKind::LessEqual,
                        TokenKind::Greater,
                        TokenKind::GreaterEqual,
                    ],
                    found: operator.to_owned(),
                    position: operator_position,
                });
            }
        };
        let jump = Instruction::jump(1, true as usize);
        let destination_index = self.next_register_index();
        let destination = Address::register(destination_index);
        let true_as_address = Address::encoded(true as usize);
        let load_true = Instruction::load(
            destination,
            true_as_address,
            OperandType::BOOLEAN,
            true as usize,
        );
        let false_as_address = Address::encoded(false as usize);
        let load_false = Instruction::load(
            destination,
            false_as_address,
            OperandType::BOOLEAN,
            false as usize,
        );
        let comparison_position = Span(left_position.0, right_position.1);

        self.emit_instruction(comparison, Type::Boolean, comparison_position);
        self.emit_instruction(jump, Type::None, comparison_position);
        self.emit_instruction(load_true, Type::Boolean, comparison_position);
        self.emit_instruction(load_false, Type::Boolean, comparison_position);

        Ok(())
    }

    fn parse_logical_binary(&mut self) -> Result<(), CompileError> {
        let is_logic_chain = matches!(
            self.get_last_operations(),
            Some([Operation::TEST, Operation::JUMP, Operation::LOAD])
        );

        let (left, left_type, left_position) =
            self.instructions
                .last()
                .cloned()
                .ok_or_else(|| CompileError::ExpectedExpression {
                    found: self.previous_token.to_owned(),
                    position: self.previous_position,
                })?;
        let left_destination = left.destination();

        if left_type != Type::Boolean {
            return Err(CompileError::ExpectedBoolean {
                found: self.previous_token.to_owned(),
                position: left_position,
            });
        }

        let operator = self.current_token;
        let operator_position = self.current_position;
        let rule = ParseRule::from(operator);
        let comparator = match operator {
            Token::DoubleAmpersand => true,
            Token::DoublePipe => false,
            _ => {
                return Err(CompileError::ExpectedTokenMultiple {
                    expected: &[TokenKind::DoubleAmpersand, TokenKind::DoublePipe],
                    found: operator.to_owned(),
                    position: operator_position,
                });
            }
        };
        let test = Instruction::test(left_destination, comparator);

        self.emit_instruction(test, Type::None, operator_position);

        let jump_index = self.instructions.len();
        let jump_position = self.current_position;

        self.advance()?;
        self.parse_sub_expression(&rule.precedence)?;

        if matches!(
            self.get_last_operations(),
            Some([
                Operation::EQUAL | Operation::LESS | Operation::LESS_EQUAL,
                Operation::JUMP,
                Operation::LOAD,
                Operation::LOAD,
            ])
        ) {
            let instruction_count = self.instructions.len();
            let load_instructions = self
                .instructions
                .get_disjoint_mut([instruction_count - 1, instruction_count - 2])
                .unwrap();

            let Load {
                operand,
                r#type,
                jump_next,
                ..
            } = Load::from(load_instructions[0].0);
            load_instructions[0].0 = Instruction::load(
                Address::register(left_destination.index),
                operand,
                r#type,
                jump_next,
            );

            let Load {
                operand,
                r#type,
                jump_next,
                ..
            } = Load::from(load_instructions[1].0);
            load_instructions[1].0 = Instruction::load(
                Address::register(left_destination.index),
                operand,
                r#type,
                jump_next,
            );
        } else {
            let (right, right_type, right_position) = self.instructions.last_mut().unwrap();

            if right_type != &Type::Boolean {
                return Err(CompileError::ExpectedBoolean {
                    found: self.previous_token.to_owned(),
                    position: *right_position,
                });
            }

            if right.yields_value() {
                right.set_destination(left_destination);
            }
        }

        let instructions_length = self.instructions.len();
        let jump_distance = instructions_length - jump_index;
        let jump = Instruction::jump(jump_distance, true as usize);

        self.instructions
            .insert(jump_index, (jump, Type::None, jump_position));

        if is_logic_chain {
            let instruction_count = self.instructions.len();
            let start_slice = instruction_count.saturating_sub(7);
            let mut comparator = comparator;
            let mut instruction_slice = &mut self.instructions[start_slice..start_slice + 4];

            while instruction_slice[2].0.operation() == Operation::JUMP {
                if comparator {
                    let new_distance = jump_distance + (instruction_count - start_slice - 4);

                    instruction_slice[2].0.set_b_field(new_distance);
                }

                instruction_slice[0].0.set_destination(left_destination);

                let start_slice = start_slice.saturating_sub(4);
                comparator = instruction_slice[1].0.a_field() != 0;
                instruction_slice = &mut self.instructions[start_slice..start_slice + 4];

                if start_slice <= 3 {
                    break;
                }
            }
        }

        Ok(())
    }

    fn parse_variable(&mut self) -> Result<(), CompileError> {
        let variable_position = self.current_position;
        let identifier = if let Token::Identifier(text) = self.current_token {
            self.advance()?;

            text
        } else {
            return Err(CompileError::ExpectedToken {
                expected: TokenKind::Identifier,
                found: self.current_token.to_owned(),
                position: variable_position,
            });
        };
        let variable_path = Path::new_at_position(identifier, variable_position)?;

        let (variable_address, variable_type, is_mutable) = {
            if let Some(local) = self.get_local(&variable_path) {
                if !self.current_block_scope.contains(&local.scope) {
                    return Err(CompileError::VariableOutOfScope {
                        identifier: identifier.to_string(),
                        position: variable_position,
                        variable_scope: local.scope,
                        access_scope: self.current_block_scope,
                    });
                }

                (local.address, local.r#type.clone(), local.is_mutable)
            } else if let Some((cell_index, _, global)) = self.globals.borrow().get_full(identifier)
            {
                let Global { r#type, is_mutable } = global;

                (Address::cell(cell_index), r#type.clone(), *is_mutable)
            } else if let Ok((item, item_position)) = { self.clone_item(&variable_path) } {
                let (local_address, item_type) =
                    self.bring_item_into_local_scope(variable_path, item, item_position);

                (local_address, item_type, false)
            } else if let CompileMode::Function { name, .. } = &self.mode
                && *name == Some(variable_path)
            {
                let destination = Address::register(self.next_register_index());
                let load = Instruction::load(
                    destination,
                    Address::function_self(),
                    OperandType::FUNCTION,
                    false as usize,
                );

                self.emit_instruction(load, Type::FunctionSelf, variable_position);

                return Ok(());
            } else if self.allow_native_functions
                && let Some(native_function) = NativeFunction::from_str(identifier)
            {
                return self.parse_call_native(native_function, variable_position);
            } else {
                return Err(CompileError::UndeclaredVariable {
                    identifier: identifier.to_string(),
                    position: variable_position,
                });
            }
        };

        if self.allow(Token::Equal)? {
            if is_mutable {
                self.parse_expression()?;

                if self
                    .instructions
                    .last()
                    .is_some_and(|(instruction, _, _)| instruction.is_math())
                {
                    let (math_instruction, _, _) = self.instructions.last_mut().unwrap();

                    math_instruction.set_a_field(variable_address.index);
                }
            } else {
                return Err(CompileError::CannotMutateImmutableVariable {
                    identifier: identifier.to_string(),
                    position: self.previous_position,
                });
            }
        }

        let operand_type = variable_type.as_operand_type();
        let destination = Address::register(self.next_register_index());
        let load = Instruction::load(destination, variable_address, operand_type, false as usize);

        self.emit_instruction(load, variable_type, self.previous_position);

        Ok(())
    }

    fn parse_type(&mut self) -> Result<Type, CompileError> {
        match self.current_token {
            Token::Bool => {
                self.advance()?;

                Ok(Type::Boolean)
            }
            Token::ByteKeyword => {
                self.advance()?;

                Ok(Type::Byte)
            }
            Token::FloatKeyword => {
                self.advance()?;

                Ok(Type::Float)
            }
            Token::Fn => self.parse_function_type(),
            Token::Int => {
                self.advance()?;

                Ok(Type::Integer)
            }
            Token::Str => {
                self.advance()?;

                Ok(Type::String)
            }
            _ => Err(CompileError::ExpectedTokenMultiple {
                expected: &[
                    TokenKind::Bool,
                    TokenKind::ByteKeyword,
                    TokenKind::FloatKeyword,
                    TokenKind::Fn,
                    TokenKind::Int,
                    TokenKind::Str,
                ],
                found: self.current_token.to_owned(),
                position: self.current_position,
            }),
        }
    }

    fn parse_function_type(&mut self) -> Result<Type, CompileError> {
        self.advance()?;

        let mut parameters = Vec::new();
        let mut return_type = Type::None;

        if self.allow(Token::LeftParenthesis)? {
            while !self.allow(Token::RightParenthesis)? {
                let parameter_type = self.parse_type()?;

                parameters.push(parameter_type);

                self.allow(Token::Comma)?;
            }
        }

        if self.allow(Token::ArrowThin)? {
            return_type = self.parse_type()?;
        }

        Ok(Type::Function(Box::new(FunctionType::new(
            Vec::new(),
            parameters,
            return_type,
        ))))
    }

    fn parse_block(&mut self) -> Result<(), CompileError> {
        self.advance()?;

        let starting_scope = self.current_block_scope;
        let starting_block = self.current_block_scope.block_index;
        let start_register = self.next_register_index();

        self.block_index += 1;
        self.current_block_scope.begin(self.block_index);

        while !self.allow(Token::RightBrace)? && !self.is_eof() {
            self.parse(Precedence::None)?;
        }

        let end_register = self.next_register_index();

        for register_index in start_register..end_register {
            let address = Address::register(register_index);

            if !self
                .locals
                .iter()
                .any(|(_, local)| local.address == address && local.scope == starting_scope)
            {
                self.reclaimable_register_indexes.push(register_index);
            }
        }

        self.current_block_scope.end(starting_block);

        Ok(())
    }

    fn parse_list(&mut self) -> Result<(), CompileError> {
        let start = self.current_position.0;

        self.advance()?;

        let mut item_type = Type::None;
        let mut first_item_register = self.next_register_index();
        let mut last_item_register = first_item_register;
        let mut instructions_to_reorder = Vec::new();

        while !self.allow(Token::RightBracket)? {
            self.parse_expression()?;
            self.allow(Token::Comma)?;

            if item_type == Type::None {
                item_type = self.get_last_instruction_type();
            } else {
                // TODO: Check if the item type the same as the previous item type
            }

            let end_item_register = self.next_register_index_without_reclaiming();
            last_item_register = end_item_register;

            if self.instructions.last().unwrap().0.yields_value() {
                let instruction_data = self.instructions.pop().unwrap();

                instructions_to_reorder.push(instruction_data);
            }
        }

        let mut destination_register = last_item_register + 1;
        let reordered_instructions_count = instructions_to_reorder.len();

        for mut instruction_data in instructions_to_reorder {
            let register_index = self.next_register_index_without_reclaiming();

            instruction_data.0.set_a_field(register_index);

            self.instructions.push(instruction_data);
        }

        if reordered_instructions_count > 0 {
            first_item_register =
                self.next_register_index_without_reclaiming() - reordered_instructions_count;
            last_item_register = first_item_register + reordered_instructions_count - 1;
            destination_register = last_item_register + 1;
        }

        let end = self.previous_position.1;
        let operand_type = match item_type {
            Type::Boolean => OperandType::LIST_BOOLEAN,
            Type::Byte => OperandType::LIST_BYTE,
            Type::Character => OperandType::LIST_CHARACTER,
            Type::Float => OperandType::LIST_FLOAT,
            Type::Integer => OperandType::LIST_INTEGER,
            Type::String => OperandType::LIST_STRING,
            Type::List(_) => OperandType::LIST_LIST,
            Type::Function(_) => OperandType::LIST_FUNCTION,
            _ => todo!(),
        };
        let list = Instruction::list(
            Address::register(destination_register),
            Address::register(first_item_register),
            Address::register(last_item_register),
            operand_type,
        );

        self.emit_instruction(list, Type::List(Box::new(item_type)), Span(start, end));

        Ok(())
    }

    fn parse_if(&mut self) -> Result<(), CompileError> {
        let previous = if self.previous_token == Token::Else {
            self.instructions.last().cloned()
        } else {
            None
        };

        self.advance()?;
        self.parse_expression()?;

        if matches!(
            self.get_last_operations(),
            Some([
                Operation::EQUAL | Operation::LESS | Operation::LESS_EQUAL,
                Operation::JUMP,
                Operation::LOAD,
                Operation::LOAD
            ]),
        ) {
            self.instructions.pop();
            self.instructions.pop();
            self.instructions.pop();
        } else {
            let address_index = match self.get_last_instruction_type() {
                Type::Boolean => self.next_register_index() - 1,
                _ => {
                    return Err(CompileError::ExpectedBoolean {
                        found: self.previous_token.to_owned(),
                        position: self.previous_position,
                    });
                }
            };
            let test = Instruction::test(Address::register(address_index), true);

            self.emit_instruction(test, Type::None, self.current_position);
        }

        let if_block_start = self.instructions.len();
        let if_block_start_position = self.current_position;

        if let Token::LeftBrace = self.current_token {
            self.parse_block()?;
        } else {
            return Err(CompileError::ExpectedToken {
                expected: TokenKind::LeftBrace,
                found: self.current_token.to_owned(),
                position: self.current_position,
            });
        }

        let if_block_end = self.instructions.len();
        let if_block_end_position = self.current_position;
        let mut if_block_distance = if_block_end - if_block_start;

        let (if_block_last_instruction, if_block_type, _) = if let Some(previous) = previous {
            if previous.0.yields_value() {
                let previous_active_register = previous.0.a_field();
                let (last_instruction, _, _) = self.instructions.last_mut().unwrap();

                last_instruction.set_a_field(previous_active_register);
            }

            previous
        } else {
            self.instructions.last().cloned().unwrap()
        };
        let if_block_type = if_block_type.clone();
        let if_block_last_instruction_destination = if_block_last_instruction.a_field();

        if let Token::Else = self.current_token {
            self.advance()?;

            if let Token::If = self.current_token {
                self.parse_if()?;
            } else if let Token::LeftBrace = self.current_token {
                self.parse_block()?;
            } else {
                return Err(CompileError::ExpectedTokenMultiple {
                    expected: &[TokenKind::If, TokenKind::LeftBrace],
                    found: self.current_token.to_owned(),
                    position: self.current_position,
                });
            }
        } else if if_block_type != Type::None {
            return Err(CompileError::IfMissingElse {
                position: Span(if_block_start_position.0, self.current_position.1),
            });
        }

        let else_block_end = self.instructions.len();
        let jump_distance = else_block_end - if_block_end;
        let (else_block_last_instruction, else_block_type, _) =
            self.instructions.last_mut().unwrap();

        else_block_last_instruction.set_a_field(if_block_last_instruction_destination);

        if let Err(conflict) = if_block_type.check(else_block_type) {
            return Err(CompileError::IfElseBranchMismatch {
                conflict,
                position: Span(if_block_start_position.0, self.current_position.1),
            });
        }

        match jump_distance {
            0 => {}
            1 => {
                if let Some([Operation::LOAD, _]) = self.get_last_operations() {
                    let load_index = self.instructions.len() - 2;
                    let (load, _, _) = self.instructions.get_mut(load_index).unwrap();

                    *load = Instruction::load(
                        load.destination(),
                        load.b_address(),
                        load.operand_type(),
                        true as usize,
                    );
                } else {
                    if_block_distance += 1;
                    let jump = Instruction::from(Jump {
                        offset: jump_distance,
                        is_positive: true as usize,
                    });

                    self.instructions
                        .insert(if_block_end, (jump, Type::None, if_block_end_position));
                }
            }
            2.. => {
                if_block_distance += 1;
                let jump = Instruction::from(Jump {
                    offset: jump_distance,
                    is_positive: true as usize,
                });

                self.instructions
                    .insert(if_block_end, (jump, Type::None, if_block_end_position));
            }
        }

        let jump = Instruction::jump(if_block_distance, true as usize);

        self.instructions
            .insert(if_block_start, (jump, Type::None, if_block_start_position));

        Ok(())
    }

    fn parse_while(&mut self) -> Result<(), CompileError> {
        self.advance()?;

        let expression_start = self.instructions.len();

        self.parse_expression()?;

        if matches!(
            self.get_last_operations(),
            Some([
                Operation::EQUAL | Operation::LESS | Operation::LESS_EQUAL,
                Operation::JUMP,
                Operation::LOAD,
                Operation::LOAD
            ]),
        ) {
            self.instructions.pop();
            self.instructions.pop();
            self.instructions.pop();
        } else {
            let test = Instruction::test(Address::register(self.next_register_index()), true);

            self.emit_instruction(test, Type::None, self.current_position);
        }

        let block_start = self.instructions.len();

        self.parse_block()?;

        let block_end = self.instructions.len();
        let jump_distance = block_end - block_start + 1;
        let jump = Instruction::from(Jump {
            offset: jump_distance,
            is_positive: true as usize,
        });

        self.instructions
            .insert(block_start, (jump, Type::None, self.current_position));

        let jump_back_distance = block_end - expression_start + 1;
        let jump_back = Instruction::from(Jump {
            offset: jump_back_distance,
            is_positive: false as usize,
        });

        self.emit_instruction(jump_back, Type::None, self.current_position);

        Ok(())
    }

    fn parse_expression(&mut self) -> Result<(), CompileError> {
        self.parse(Precedence::None)?;

        let expression_type = self.get_last_instruction_type();

        if expression_type == Type::None || self.instructions.is_empty() {
            return Err(CompileError::ExpectedExpression {
                found: self.previous_token.to_owned(),
                position: self.current_position,
            });
        }

        self.previous_expression_end = self.instructions.len() - 1;

        Ok(())
    }

    fn parse_sub_expression(&mut self, precedence: &Precedence) -> Result<(), CompileError> {
        self.parse(precedence.increment())?;

        let expression_type = self.get_last_instruction_type();

        if expression_type == Type::None || self.instructions.is_empty() {
            return Err(CompileError::ExpectedExpression {
                found: self.previous_token.to_owned(),
                position: self.current_position,
            });
        }

        Ok(())
    }

    fn parse_return(&mut self) -> Result<(), CompileError> {
        let start = self.current_position.0;

        self.advance()?;

        let (should_return_value, return_register, operand_type) =
            if matches!(self.current_token, Token::Semicolon | Token::RightBrace) {
                self.update_return_type(Type::None)?;

                (false, 0, OperandType::default())
            } else {
                self.parse_expression()?;

                let expression_type = self.get_last_instruction_type();
                let operand_type = expression_type.as_operand_type();
                let return_register = self.next_register_index();

                self.update_return_type(expression_type)?;

                (true, return_register, operand_type)
            };
        let end = self.current_position.1;
        let r#return = Instruction::r#return(
            should_return_value as usize,
            Address::register(return_register),
            operand_type,
        );

        self.emit_instruction(r#return, Type::None, Span(start, end));

        let instruction_length = self.instructions.len();

        for (index, (instruction, _, _)) in self.instructions.iter_mut().enumerate() {
            if instruction.operation() == Operation::JUMP {
                let Jump {
                    offset,
                    is_positive,
                } = Jump::from(*instruction);

                if is_positive != 0 && offset + index == instruction_length - 1 {
                    *instruction = Instruction::jump(offset + 1, true as usize);
                }
            }
        }

        Ok(())
    }

    fn parse_implicit_return(&mut self) -> Result<(), CompileError> {
        if matches!(self.get_last_operation(), Some(Operation::LOAD)) {
            let previous_is_comparison = matches!(
                self.get_last_operations(),
                Some([
                    Operation::EQUAL | Operation::LESS | Operation::LESS_EQUAL,
                    Operation::JUMP,
                    Operation::LOAD,
                    Operation::LOAD,
                ])
            );
            let previous_is_logic = matches!(
                self.get_last_operations(),
                Some([
                    Operation::LOAD,
                    Operation::TEST,
                    Operation::JUMP,
                    Operation::LOAD,
                ])
            ) || matches!(
                self.get_last_operations(),
                Some([
                    Operation::LOAD,
                    Operation::TEST,
                    Operation::JUMP,
                    Operation::LOAD,
                    Operation::LOAD,
                ])
            );

            let (load_instruction, expression_type, position) = self.instructions.pop().unwrap();
            let Load {
                destination,
                operand,
                r#type,
                ..
            } = Load::from(load_instruction);
            let should_return_value = expression_type != Type::None;
            let return_address = if !should_return_value {
                self.instructions
                    .push((load_instruction, expression_type.clone(), position));

                Address::default()
            } else if previous_is_comparison || previous_is_logic {
                self.instructions
                    .push((load_instruction, expression_type.clone(), position));

                destination
            } else {
                operand
            };

            let r#return =
                Instruction::r#return(should_return_value as usize, return_address, r#type);

            self.update_return_type(expression_type.clone())?;
            self.emit_instruction(r#return, expression_type, self.current_position);
        } else if matches!(self.get_last_operation(), Some(Operation::RETURN))
            || matches!(
                self.get_last_operations(),
                Some([Operation::RETURN, Operation::JUMP])
            )
        {
            // Do nothing if the last instruction is a return or a return followed by a jump
        } else if self.allow(Token::Semicolon)? {
            let r#return =
                Instruction::r#return(false as usize, Address::default(), OperandType::NONE);

            self.update_return_type(Type::None)?;
            self.emit_instruction(r#return, Type::None, self.current_position);
        } else if let Some((last_instruction, last_instruction_type, _)) = self.instructions.last()
        {
            let operand_type = last_instruction_type.as_operand_type();
            let return_value_address = last_instruction.destination();
            let should_return_value = operand_type != OperandType::NONE;
            let r#return = Instruction::r#return(
                should_return_value as usize,
                return_value_address,
                operand_type,
            );

            self.update_return_type(last_instruction_type.clone())?;
            self.emit_instruction(r#return, Type::None, self.current_position);
        }

        Ok(())
    }

    fn parse_let(&mut self) -> Result<(), CompileError> {
        self.advance()?;

        let is_mutable = self.allow(Token::Mut)?;
        let is_cell = self.allow(Token::Cell)?;
        let path = if let Token::Identifier(text) = self.current_token {
            self.advance()?;

            Path::new_at_position(text, self.previous_position)?
        } else {
            return Err(CompileError::ExpectedToken {
                expected: TokenKind::Identifier,
                found: self.current_token.to_owned(),
                position: self.current_position,
            });
        };
        let explicit_type_and_position = if self.allow(Token::Colon)? {
            let start = self.current_position.0;
            let r#type = self.parse_type()?;
            let end = self.current_position.1;

            Some((r#type, Span(start, end)))
        } else {
            None
        };

        self.expect(Token::Equal)?;
        self.parse_expression()?;
        self.allow(Token::Semicolon)?;

        self.previous_statement_end = self.instructions.len() - 1;
        self.previous_expression_end = self.instructions.len() - 1;

        let (mut last_instruction, last_instruction_type, last_instruction_position) = self
            .instructions
            .pop()
            .ok_or_else(|| CompileError::ExpectedExpression {
                found: self.previous_token.to_owned(),
                position: self.previous_position,
            })?;
        let r#type = if let Some((r#type, position)) = explicit_type_and_position {
            r#type.check(&last_instruction_type).map_err(|conflict| {
                CompileError::LetStatementTypeConflict {
                    conflict,
                    expected_position: position,
                    actual_position: last_instruction_position,
                }
            })?;

            r#type
        } else {
            last_instruction_type.clone()
        };
        let address = last_instruction.destination();

        if is_cell {
            let cell_index = self.declare_global(path, r#type, is_mutable);

            last_instruction.set_destination(Address::cell(cell_index));
        } else {
            self.declare_local(path, address, r#type, is_mutable, self.current_block_scope);
        }

        self.instructions
            .push((last_instruction, Type::None, last_instruction_position));

        Ok(())
    }

    fn parse_function(&mut self) -> Result<(), CompileError> {
        if let Token::Fn = self.current_token {
            self.advance()?;
        }

        let path = if let Token::Identifier(text) = self.current_token {
            self.advance()?;

            let path = Path::new(text).ok_or_else(|| CompileError::InvalidPath {
                found: text.to_string(),
                position: self.current_position,
            })?;

            Some(path)
        } else {
            None
        };
        let mut function_compiler = if self.current_token == Token::LeftParenthesis {
            let mode = CompileMode::Function { name: path.clone() };
            let mut compiler = ChunkCompiler::new(
                self.lexer,
                mode,
                self.current_item_scope.clone(),
                self.main_module.clone(),
                self.globals.clone(),
                self.prototypes.clone(),
            )?; // This will consume the parenthesis

            compiler.prototype_index = self.constants.len();
            compiler.allow_native_functions = self.allow_native_functions;

            compiler
        } else {
            return Err(CompileError::ExpectedToken {
                expected: TokenKind::LeftParenthesis,
                found: self.current_token.to_owned(),
                position: self.current_position,
            });
        };
        let mut value_parameters = Vec::with_capacity(3);

        while !function_compiler.allow(Token::RightParenthesis)? {
            let is_mutable = function_compiler.allow(Token::Mut)?;
            let parameter = if let Token::Identifier(text) = function_compiler.current_token {
                function_compiler.advance()?;

                Path::new(text).ok_or_else(|| CompileError::InvalidPath {
                    found: text.to_string(),
                    position: function_compiler.current_position,
                })?
            } else {
                return Err(CompileError::ExpectedToken {
                    expected: TokenKind::Identifier,
                    found: function_compiler.current_token.to_owned(),
                    position: function_compiler.current_position,
                });
            };

            function_compiler.expect(Token::Colon)?;

            let r#type = function_compiler.parse_type()?;
            let local_register_index = function_compiler.next_register_index();
            let address = Address::register(local_register_index);

            function_compiler.declare_local(
                parameter,
                address,
                r#type.clone(),
                is_mutable,
                function_compiler.current_block_scope,
            );
            value_parameters.push(r#type);
            function_compiler.allow(Token::Comma)?;
        }

        let return_type = if function_compiler.allow(Token::ArrowThin)? {
            function_compiler.parse_type()?
        } else {
            Type::None
        };
        function_compiler.r#type = FunctionType::new([], value_parameters, return_type);

        function_compiler.expect(Token::LeftBrace)?;
        function_compiler.compile()?;
        function_compiler.expect(Token::RightBrace)?;

        self.previous_token = function_compiler.previous_token;
        self.previous_position = function_compiler.previous_position;
        self.current_token = function_compiler.current_token;
        self.current_position = function_compiler.current_position;

        self.lexer.skip_to(self.current_position.1);

        let register_count = function_compiler.next_register_index_without_reclaiming();
        let instructions = function_compiler
            .instructions
            .into_iter()
            .map(|(instruction, _, _)| instruction)
            .collect();
        let locals = function_compiler.locals.into_iter().collect();
        let chunk = Chunk {
            name: path.clone(),
            r#type: function_compiler.r#type,
            instructions,
            constants: function_compiler.constants,
            locals,
            register_count,
            prototype_index: function_compiler.prototype_index,
        };
        let prototype_address = Address::constant(chunk.prototype_index);
        let r#type = Type::Function(Box::new(chunk.r#type.clone()));

        if let Some(identifier) = path {
            self.declare_local(
                identifier,
                prototype_address,
                r#type.clone(),
                false,
                self.current_block_scope,
            );
        }

        self.prototypes.borrow_mut().push(Arc::new(chunk));

        Ok(())
    }

    fn parse_call(&mut self) -> Result<(), CompileError> {
        let start = self.previous_position.0;

        self.advance()?;

        let (function, return_type, function_index) = {
            let prototypes = self.prototypes.borrow();
            let function = prototypes
                .last()
                .ok_or_else(|| CompileError::ExpectedFunction {
                    found: self.previous_token.to_owned(),
                    position: self.previous_position,
                })?;
            let index = self.prototypes.borrow().len() - 1;

            (function.clone(), function.r#type.return_type.clone(), index)
        };
        let argument_count = function.r#type.value_parameters.len();
        let mut argument_registers = Vec::with_capacity(argument_count);

        while !self.allow(Token::RightParenthesis)? {
            self.parse_expression()?;
            self.allow(Token::Comma)?;

            let argument_register = self.next_register_index() - 1;

            argument_registers.push(argument_register);
        }

        let destination_index = self.next_register_index();
        let correct_argument_range = (destination_index - argument_count)..destination_index;
        let arguments_are_in_correct_registers = argument_registers
            .iter()
            .zip(correct_argument_range)
            .all(|(argument_register, expected_register)| *argument_register == expected_register);

        if !arguments_are_in_correct_registers {
            todo!()
        }

        let end = self.current_position.1;
        let return_operand_type = return_type.as_operand_type();
        let destination = Address::register(self.next_register_index());
        let call = Instruction::call(
            destination,
            Address::constant(function_index),
            argument_count,
            return_operand_type,
        );

        self.emit_instruction(call, return_type, Span(start, end));

        Ok(())
    }

    fn parse_call_native(
        &mut self,
        function: NativeFunction,
        start: Span,
    ) -> Result<(), CompileError> {
        let mut type_arguments = Vec::new();

        if self.allow(Token::Less)? {
            while !self.allow(Token::Greater)? {
                let r#type = self.parse_type()?;

                type_arguments.push(r#type);

                self.allow(Token::Comma)?;
            }
        }

        self.expect(Token::LeftParenthesis)?;

        let argument_count = function.r#type().value_parameters.len();
        let mut argument_registers = Vec::with_capacity(argument_count);

        while !self.allow(Token::RightParenthesis)? {
            self.parse_expression()?;
            self.allow(Token::Comma)?;

            let argument_register = self.next_register_index() - 1;

            argument_registers.push(argument_register);
        }

        let destination_index = self.next_register_index();
        let correct_argument_range = (destination_index - argument_count)..destination_index;
        let arguments_are_in_correct_registers = argument_registers
            .iter()
            .zip(correct_argument_range)
            .all(|(argument_register, expected_register)| *argument_register == expected_register);

        if !arguments_are_in_correct_registers {
            todo!()
        }

        let end = self.current_position.1;
        let return_type = function.r#type().return_type.clone();
        let destination = Address::register(destination_index);
        let call_native = Instruction::call_native(destination, function, argument_count);

        self.emit_instruction(call_native, return_type, Span(start.0, end));

        Ok(())
    }

    fn parse_semicolon(&mut self) -> Result<(), CompileError> {
        let (_, last_expression_type, _) =
            self.instructions
                .last_mut()
                .ok_or_else(|| CompileError::ExpectedExpression {
                    found: self.previous_token.to_owned(),
                    position: self.previous_position,
                })?;

        *last_expression_type = Type::None;

        self.advance()?;

        Ok(())
    }

    fn parse_mod(&mut self) -> Result<(), CompileError> {
        let loggging = span!(Level::TRACE, "Module");
        let _span_guard = loggging.enter();
        let start = self.current_position.0;

        self.advance()?;

        let name = if let Token::Identifier(text) = self.current_token {
            self.advance()?;

            Path::new_at_position(text, self.previous_position)?
        } else {
            return Err(CompileError::ExpectedToken {
                expected: TokenKind::Identifier,
                found: self.current_token.to_owned(),
                position: self.current_position,
            });
        };

        let old_mode = replace(
            &mut self.mode,
            CompileMode::Module {
                name,
                module: Module::new(),
            },
        );

        self.expect(Token::LeftBrace)?;
        self.parse_items()?;
        self.expect(Token::RightBrace)?;

        let end = self.previous_position.1;
        let position = Span(start, end);

        if let CompileMode::Module {
            name: new_module_path,
            module: new_module,
        } = replace(&mut self.mode, old_mode)
        {
            self.main_module.borrow_mut().items.insert(
                new_module_path.clone(),
                (Item::Module(new_module), position),
            );

            self.current_item_scope.insert(new_module_path);
        }

        Ok(())
    }

    fn parse_items(&mut self) -> Result<(), CompileError> {
        loop {
            match self.current_token {
                Token::Const => self.parse_const()?,
                Token::Fn => {
                    let function_start = self.current_position.0;

                    self.parse_function()?;

                    let function_position = Span(function_start, self.current_position.1);
                    let prototype = self.prototypes.borrow_mut().pop().unwrap();

                    if let CompileMode::Module { module, .. } = &mut self.mode
                        && let Some(path) = &prototype.name
                    {
                        module
                            .items
                            .insert(path.clone(), (Item::Function(prototype), function_position));
                    }
                }
                Token::Mod => self.parse_mod()?,
                _ => break Ok(()),
            }
        }
    }

    fn parse_const(&mut self) -> Result<(), CompileError> {
        let start = self.current_position.0;

        self.advance()?;

        let path = if let Token::Identifier(text) = self.current_token {
            self.advance()?;

            Path::new_at_position(text, self.previous_position)?
        } else {
            return Err(CompileError::ExpectedToken {
                expected: TokenKind::Identifier,
                found: self.current_token.to_owned(),
                position: self.current_position,
            });
        };

        self.expect(Token::Colon)?;

        let r#type = self.parse_type()?;

        self.expect(Token::Equal)?;

        let value = match r#type.as_concrete_type() {
            ConcreteType::Boolean => {
                let boolean = if let Token::Boolean(text) = self.current_token {
                    self.advance()?;
                    self.parse_boolean_value(text)
                } else {
                    return Err(CompileError::ExpectedToken {
                        expected: TokenKind::Boolean,
                        found: self.current_token.to_owned(),
                        position: self.current_position,
                    });
                };

                Value::boolean(boolean)
            }
            ConcreteType::Byte => {
                let byte = if let Token::Byte(text) = self.current_token {
                    self.advance()?;
                    self.parse_byte_value(text)?
                } else {
                    return Err(CompileError::ExpectedToken {
                        expected: TokenKind::Byte,
                        found: self.current_token.to_owned(),
                        position: self.current_position,
                    });
                };

                Value::byte(byte)
            }
            ConcreteType::Character => {
                let character = if let Token::Character(character) = self.current_token {
                    self.advance()?;

                    character
                } else {
                    return Err(CompileError::ExpectedToken {
                        expected: TokenKind::Character,
                        found: self.current_token.to_owned(),
                        position: self.current_position,
                    });
                };

                Value::character(character)
            }
            ConcreteType::Float => {
                let float = if let Token::Float(text) = self.current_token {
                    self.advance()?;
                    self.parse_float_value(text)?
                } else {
                    return Err(CompileError::ExpectedToken {
                        expected: TokenKind::Float,
                        found: self.current_token.to_owned(),
                        position: self.current_position,
                    });
                };

                Value::float(float)
            }
            ConcreteType::Integer => {
                let integer = if let Token::Integer(text) = self.current_token {
                    self.advance()?;
                    self.parse_integer_value(text)
                } else {
                    return Err(CompileError::ExpectedToken {
                        expected: TokenKind::Integer,
                        found: self.current_token.to_owned(),
                        position: self.current_position,
                    });
                };

                Value::integer(integer)
            }
            ConcreteType::String => {
                let string = if let Token::String(text) = self.current_token {
                    self.advance()?;

                    String::from(text)
                } else {
                    return Err(CompileError::ExpectedToken {
                        expected: TokenKind::String,
                        found: self.current_token.to_owned(),
                        position: self.current_position,
                    });
                };

                Value::string(string)
            }
            _ => todo!(),
        };
        let end = self.current_position.1;
        let position = Span(start, end);
        let constant = Item::Constant { value, r#type };

        match &mut self.mode {
            CompileMode::Module { module, .. } => {
                module.items.insert(path, (constant, position));
            }
            _ => {
                self.main_module
                    .borrow_mut()
                    .items
                    .insert(path, (constant, position));
            }
        }

        self.allow(Token::Semicolon)?;

        Ok(())
    }

    fn parse_use(&mut self) -> Result<(), CompileError> {
        self.advance()?;

        let path = if let Token::Identifier(text) = self.current_token {
            self.advance()?;

            Path::new_at_position(text, self.previous_position)?
        } else {
            return Err(CompileError::ExpectedToken {
                expected: TokenKind::Identifier,
                found: self.current_token.to_owned(),
                position: self.current_position,
            });
        };

        self.allow(Token::Semicolon)?;

        let (item, item_position) = self.clone_item(&path)?;

        self.bring_item_into_local_scope(path.item(), item, item_position);

        Ok(())
    }

    fn bring_item_into_local_scope(
        &mut self,
        item_name: Path,
        item: Item,
        item_position: Span,
    ) -> (Address, Type) {
        match item {
            Item::Constant { value, r#type } => {
                let destination = Address::register(self.next_register_index());
                let operand = match value {
                    Value::Boolean(boolean) => Address::constant(boolean as usize),
                    Value::Byte(byte) => Address::constant(byte as usize),
                    Value::Character(character) => {
                        let value = Value::character(character);
                        let constant_index = self.push_constant_or_get_index(value);

                        Address::new(constant_index, MemoryKind::CONSTANT)
                    }
                    Value::Float(float) => {
                        let value = Value::float(float);
                        let constant_index = self.push_constant_or_get_index(value);

                        Address::new(constant_index, MemoryKind::CONSTANT)
                    }
                    Value::Integer(integer) => {
                        let value = Value::integer(integer);
                        let constant_index = self.push_constant_or_get_index(value);

                        Address::new(constant_index, MemoryKind::CONSTANT)
                    }
                    Value::String(string) => {
                        let value = Value::string(string);
                        let constant_index = self.push_constant_or_get_index(value);

                        Address::new(constant_index, MemoryKind::CONSTANT)
                    }
                    Value::List(list) => match list {
                        List::Boolean(booleans) => {
                            let value = Value::boolean_list(booleans);
                            let constant_index = self.push_constant_or_get_index(value);

                            Address::new(constant_index, MemoryKind::CONSTANT)
                        }
                        List::Byte(bytes) => {
                            let value = Value::byte_list(bytes);
                            let constant_index = self.push_constant_or_get_index(value);

                            Address::new(constant_index, MemoryKind::CONSTANT)
                        }
                        List::Character(characters) => {
                            let value = Value::character_list(characters);
                            let constant_index = self.push_constant_or_get_index(value);

                            Address::new(constant_index, MemoryKind::CONSTANT)
                        }
                        List::Float(floats) => {
                            let value = Value::float_list(floats);
                            let constant_index = self.push_constant_or_get_index(value);

                            Address::new(constant_index, MemoryKind::CONSTANT)
                        }
                        List::Integer(integers) => {
                            let value = Value::integer_list(integers);
                            let constant_index = self.push_constant_or_get_index(value);

                            Address::new(constant_index, MemoryKind::CONSTANT)
                        }
                        List::String(strings) => {
                            let value = Value::string_list(strings);
                            let constant_index = self.push_constant_or_get_index(value);

                            Address::new(constant_index, MemoryKind::CONSTANT)
                        }
                        List::List(lists) => {
                            let value = Value::list_list(lists);
                            let constant_index = self.push_constant_or_get_index(value);

                            Address::new(constant_index, MemoryKind::CONSTANT)
                        }
                        List::Function(functions) => {
                            let value = Value::function_list(functions);
                            let constant_index = self.push_constant_or_get_index(value);

                            Address::new(constant_index, MemoryKind::CONSTANT)
                        }
                    },
                    _ => todo!("Handle other constant types in use statement"),
                };
                let instruction = Instruction::load(
                    destination,
                    operand,
                    r#type.as_operand_type(),
                    false as usize,
                );

                self.emit_instruction(instruction, Type::None, item_position);
                self.declare_local(
                    item_name,
                    destination,
                    r#type.clone(),
                    false,
                    self.current_block_scope,
                );

                (destination, r#type)
            }
            Item::Function(prototype) => {
                let r#type = Type::Function(Box::new(prototype.r#type.clone()));
                let prototype_index = {
                    let mut prototypes = self.prototypes.borrow_mut();
                    let prototype_index = prototypes.len();

                    prototypes.push(prototype);

                    prototype_index
                };
                let address = Address::constant(prototype_index);

                self.declare_local(
                    item_name,
                    address,
                    r#type.clone(),
                    false,
                    self.current_block_scope,
                );

                (address, r#type)
            }
            _ => todo!("Handle other item types"),
        }
    }

    fn parse_str(&mut self) -> Result<(), CompileError> {
        self.advance()?;
        self.expect(Token::Dot)?;

        let identifier = if let Token::Identifier(text) = self.current_token {
            self.advance()?;

            text
        } else {
            return Err(CompileError::ExpectedToken {
                expected: TokenKind::Identifier,
                found: self.current_token.to_owned(),
                position: self.current_position,
            });
        };

        match identifier {
            "from_int" => {
                let aliased_path = Path::new("std::convert::int_to_str").unwrap();
                let (item, item_position) = self.clone_item(&aliased_path)?;
                let (variable_address, item_type) =
                    self.bring_item_into_local_scope(aliased_path, item, item_position);
                let destination = Address::register(self.next_register_index());
                let load = Instruction::load(
                    destination,
                    variable_address,
                    OperandType::FUNCTION,
                    false as usize,
                );

                self.emit_instruction(load, item_type, self.previous_position);
                self.parse_call()?;
            }
            _ => {
                return Err(CompileError::UnknownItem {
                    item_name: identifier.to_string(),
                    position: self.previous_position,
                });
            }
        }

        Ok(())
    }

    fn expect_expression(&mut self) -> Result<(), CompileError> {
        Err(CompileError::ExpectedExpression {
            found: self.current_token.to_owned(),
            position: self.current_position,
        })
    }

    fn parse(&mut self, precedence: Precedence) -> Result<(), CompileError> {
        if let Some(prefix_parser) = ParseRule::from(self.current_token).prefix {
            debug!("{} is prefix", self.current_token,);

            prefix_parser(self)?;
        }

        let mut infix_rule = ParseRule::from(self.current_token);

        while precedence <= infix_rule.precedence {
            if let Some(infix_parser) = infix_rule.infix {
                debug!(
                    "{} is infix with precedence {}",
                    self.current_token, infix_rule.precedence
                );

                if self.current_token == Token::Equal {
                    return Err(CompileError::InvalidAssignmentTarget {
                        found: self.current_token.to_owned(),
                        position: self.current_position,
                    });
                }

                infix_parser(self)?;
            } else {
                break;
            }

            infix_rule = ParseRule::from(self.current_token);
        }

        Ok(())
    }
}
