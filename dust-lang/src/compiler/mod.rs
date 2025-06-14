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
mod module;
mod parse_rule;
mod path;
mod standard_library;
mod type_checks;

pub use compile_error::CompileError;
use compile_mode::CompileMode;
use global::Global;
pub use item::Item;
pub use module::{Module, find_item};
use parse_rule::{ParseRule, Precedence};
pub use path::Path;
pub use standard_library::generate_standard_library;
use tracing::{Level, debug, info, span, trace};
use type_checks::{check_math_type, check_math_types};

use std::{
    collections::{HashMap, HashSet},
    iter::repeat,
    mem::replace,
    sync::Arc,
};

use crate::{
    Address, BlockScope, Chunk, ConcreteValue, DustError, DustString, FunctionType, Instruction,
    Lexer, Local, NativeFunction, Operation, Span, Token, TokenKind, Type,
    dust_crate::{DustCrate, Program},
    instruction::{Jump, Load, MemoryKind, OperandType},
    r#type::TypeKind,
};

pub const DEFAULT_REGISTER_COUNT: usize = 4;

/// Compiles the input and returns a chunk.
///
/// # Example
///
/// ```
/// # use dust_lang::compile;
/// let source = "40 + 2 == 42";
/// let chunk = compile(source).unwrap();
///
/// assert_eq!(chunk.instructions().len(), 6);
/// ```
pub fn compile(source: &str) -> Result<Chunk, DustError> {
    let compiler = Compiler::new(source, "main");
    let Program { main_chunk, .. } = compiler
        .compile_program()
        .map_err(|error| DustError::compile(error, source))?;

    Ok(main_chunk)
}

/// The Dust compiler assembles a [`Chunk`] for the Dust VM. Any unrecognized symbols, disallowed
/// syntax or conflicting type usage will result in an error.
///
/// See the [`compile`] function an example of how to create and use a Compiler.
#[derive(Debug)]
pub struct Compiler<'a> {
    main_module: Module<'a>,
    globals: HashMap<DustString, Global>,
    source: &'a str,
    crate_name: &'a str,
}

impl<'a> Compiler<'a> {
    pub fn new(source: &'a str, crate_name: &'a str) -> Self {
        let dust_crate = Module::new();
        let globals = HashMap::new();

        Self {
            main_module: dust_crate,
            globals,
            source,
            crate_name,
        }
    }

    pub fn compile_library(mut self) -> Result<DustCrate<'a>, CompileError> {
        self.compile()?;

        Ok(DustCrate::Library(self.main_module))
    }

    pub fn compile_program(mut self) -> Result<Program<'a>, CompileError> {
        let main_chunk = self.compile()?;

        Ok(Program {
            main_chunk,
            main_module: replace(&mut self.main_module, Module::new()),
            cell_count: DEFAULT_REGISTER_COUNT as u16,
        })
    }

    fn compile(&mut self) -> Result<Chunk, CompileError> {
        let logging = span!(Level::INFO, "Compile");
        let _enter = logging.enter();

        generate_standard_library(&mut self.main_module)?;

        let lexer = Lexer::new(self.source);
        let mut compiler = ChunkCompiler::<DEFAULT_REGISTER_COUNT>::new_main(
            lexer,
            Some(self.crate_name),
            &mut self.main_module,
            &mut self.globals,
        )?;

        compiler.compile()?;

        Ok(compiler.finish())
    }
}

#[derive(Debug)]
struct ChunkCompiler<'a, 'paths, 'src, const REGISTER_COUNT: usize = DEFAULT_REGISTER_COUNT> {
    /// Indication of what the compiler will produce when it finishes. This value should never be
    /// mutated.
    mode: CompileMode<'paths>,

    /// Used to get tokens for the compiler.
    lexer: Lexer<'src>,

    /// Type of the function being compiled. This is assigned to the chunk when [`Compiler::finish`]
    /// is called.
    r#type: FunctionType,

    /// Instructions, along with their types and positions, that have been compiled. The
    /// instructions and positions are assigned to the chunk when [`Compiler::finish`] is called.
    /// The types are discarded after compilation.
    instructions: Vec<(Instruction, Type, Span)>,

    /// Character constants that have been compiled. These are assigned to the chunk when
    /// [`Compiler::finish`] is called.
    character_constants: Vec<char>,

    /// Float constants that have been compiled. These are assigned to the chunk when
    /// [`Compiler::finish`] is called.
    float_constants: Vec<f64>,

    /// Integer constants that have been compiled. These are assigned to the chunk when
    /// [`Compiler::finish`] is called.
    integer_constants: Vec<i64>,

    /// String constants that have been compiled. These are assigned to the chunk when
    /// [`Compiler::finish`] is called.
    string_constants: Vec<DustString>,

    /// Functions that have been compiled. These are assigned to the chunk when [`Compiler::finish`]
    /// is called.
    prototypes: Vec<Arc<Chunk>>,

    /// Block-local variables and their types. The locals are assigned to the chunk when
    /// [`Compiler::finish`] is called. The types are discarded after compilation.
    locals: Vec<Local>,

    globals: &'a mut HashMap<DustString, Global>,

    /// Arguments for each function call.
    arguments: Vec<(Vec<(Address, TypeKind)>, Vec<Type>)>,

    minimum_boolean_memory_index: u16,
    minimum_byte_memory_index: u16,
    minimum_character_memory_index: u16,
    minimum_float_memory_index: u16,
    minimum_integer_memory_index: u16,
    minimum_string_memory_index: u16,
    minimum_list_memory_index: u16,
    minimum_function_memory_index: u16,

    reclaimable_memory: Vec<(Address, TypeKind)>,

    /// Index of the current block. This is used to determine the scope of locals and is incremented
    /// when a new block is entered.
    block_index: u8,

    /// This is mutated during compilation to match the current block and is used to test if a local
    /// variable is in scope.
    current_block_scope: BlockScope,

    /// This is mutated during compilation as items are brought into scope by `use` statements or
    /// are invoked by their full path. It is used to test if an item is in scope.
    current_item_scope: HashSet<Path<'paths>>,

    main_module: &'a mut Module<'paths>,

    /// Index of the Chunk in its parent's prototype list. This is set to 0 for the main chunk but
    /// that value is never read because the main chunk is not a callable function.
    prototype_index: u16,

    previous_statement_end: usize,
    previous_expression_end: usize,

    current_token: Token<'src>,
    current_position: Span,
    previous_token: Token<'src>,
    previous_position: Span,

    allow_native_functions: bool,
}

impl<'a, 'paths, 'src, const REGISTER_COUNT: usize> ChunkCompiler<'a, 'paths, 'src, REGISTER_COUNT>
where
    'src: 'a + 'paths,
{
    /// Creates a new compiler.
    pub fn new_main(
        mut lexer: Lexer<'src>,
        name: Option<&'paths str>,
        main_module: &'a mut Module<'paths>,
        globals: &'a mut HashMap<DustString, Global>,
    ) -> Result<Self, CompileError> {
        let mode = CompileMode::Main { name };
        let (current_token, current_position) = lexer.next_token()?;
        let mut current_item_scope = HashSet::with_capacity(1);
        let path = Path::new_borrowed("main").unwrap();

        current_item_scope.insert(path);

        Ok(ChunkCompiler {
            mode,
            r#type: FunctionType::default(),
            instructions: Vec::new(),
            character_constants: Vec::new(),
            float_constants: Vec::new(),
            integer_constants: Vec::new(),
            string_constants: Vec::new(),
            locals: Vec::new(),
            globals,
            prototypes: Vec::new(),
            arguments: Vec::new(),
            lexer,
            minimum_byte_memory_index: 0,
            minimum_boolean_memory_index: 0,
            minimum_character_memory_index: 0,
            minimum_float_memory_index: 0,
            minimum_integer_memory_index: 0,
            minimum_string_memory_index: 0,
            minimum_list_memory_index: 0,
            minimum_function_memory_index: 0,
            reclaimable_memory: Vec::new(),
            block_index: 0,
            current_block_scope: BlockScope::default(),
            current_item_scope,
            main_module,
            prototype_index: 0,
            current_token,
            current_position,
            previous_token: Token::Eof,
            previous_position: Span(0, 0),
            previous_statement_end: 0,
            previous_expression_end: 0,
            allow_native_functions: false,
        })
    }

    pub fn new_function(
        mut lexer: Lexer<'src>,
        name: Option<&'paths str>,
        item_scope: HashSet<Path<'paths>>,
        main_module: &'a mut Module<'paths>,
        globals: &'a mut HashMap<DustString, Global>,
    ) -> Result<Self, CompileError> {
        let mode = CompileMode::Function { name };
        let (current_token, current_position) = lexer.next_token()?;

        Ok(ChunkCompiler {
            mode,
            r#type: FunctionType::default(),
            instructions: Vec::new(),
            character_constants: Vec::new(),
            float_constants: Vec::new(),
            integer_constants: Vec::new(),
            string_constants: Vec::new(),
            locals: Vec::new(),
            globals,
            prototypes: Vec::new(),
            arguments: Vec::new(),
            lexer,
            minimum_byte_memory_index: 0,
            minimum_boolean_memory_index: 0,
            minimum_character_memory_index: 0,
            minimum_float_memory_index: 0,
            minimum_integer_memory_index: 0,
            minimum_string_memory_index: 0,
            minimum_list_memory_index: 0,
            minimum_function_memory_index: 0,
            reclaimable_memory: Vec::new(),
            block_index: 0,
            current_block_scope: BlockScope::default(),
            current_item_scope: item_scope,
            main_module,
            prototype_index: 0,
            current_token,
            current_position,
            previous_token: Token::Eof,
            previous_position: Span(0, 0),
            previous_statement_end: 0,
            previous_expression_end: 0,
            allow_native_functions: false,
        })
    }

    pub fn new_module(
        mut lexer: Lexer<'src>,
        name: &'paths str,
        main_module: &'a mut Module<'paths>,
        globals: &'a mut HashMap<DustString, Global>,
    ) -> Result<Self, CompileError> {
        let mode = CompileMode::Module {
            name,
            module: Module::new(),
        };
        let (current_token, current_position) = lexer.next_token()?;
        let mut current_item_scope = HashSet::with_capacity(1);

        current_item_scope.insert(Path::new_borrowed(name).unwrap());

        Ok(ChunkCompiler {
            mode,
            r#type: FunctionType::default(),
            instructions: Vec::new(),
            character_constants: Vec::new(),
            float_constants: Vec::new(),
            integer_constants: Vec::new(),
            string_constants: Vec::new(),
            locals: Vec::new(),
            globals,
            prototypes: Vec::new(),
            arguments: Vec::new(),
            lexer,
            minimum_byte_memory_index: 0,
            minimum_boolean_memory_index: 0,
            minimum_character_memory_index: 0,
            minimum_float_memory_index: 0,
            minimum_integer_memory_index: 0,
            minimum_string_memory_index: 0,
            minimum_list_memory_index: 0,
            minimum_function_memory_index: 0,
            reclaimable_memory: Vec::new(),
            block_index: 0,
            current_block_scope: BlockScope::default(),
            current_item_scope,
            main_module,
            prototype_index: 0,
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
        for (top_level_path, (_, _)) in self.main_module.items.iter() {
            self.current_item_scope.insert(top_level_path.clone());
        }

        let logging = span!(Level::INFO, "Compile");
        let _enter = logging.enter();

        if self.mode.is_module() {
            let loggging = span!(Level::TRACE, "Module");
            let _span_guard = loggging.enter();

            self.parse_items()?;

            if let CompileMode::Module { module, .. } = &mut self.mode {
                self.main_module.items.extend(module.items.drain());
            }

            return Ok(());
        }

        info!(
            "Begin chunk with `{}` at {}",
            self.current_token, self.current_position
        );

        while !matches!(self.current_token, Token::Eof | Token::RightBrace) {
            self.parse(Precedence::None)?;
        }

        self.parse_implicit_return()?;
        self.optimize_instructions();

        if self.instructions.is_empty() {
            let r#return = Instruction::r#return(false, Address::default(), OperandType::NONE);

            self.emit_instruction(r#return, Type::None, self.current_position);
        }

        info!("End chunk");

        Ok(())
    }

    /// Creates a new chunk with the compiled data.
    pub fn finish(self) -> Chunk {
        let mut boolean_memory_length = 0;
        let mut byte_memory_length = 0;
        let mut character_memory_length = 0;
        let mut float_memory_length = 0;
        let mut integer_memory_length = 0;
        let mut string_memory_length = 0;
        let mut list_memory_length = 0;
        let mut function_memory_length = 0;
        let mut instructions = Vec::with_capacity(self.instructions.len());
        let mut positions = Vec::with_capacity(self.instructions.len());

        for (instruction, r#type, position) in self.instructions {
            if instruction.yields_value() {
                match r#type {
                    Type::Boolean => {
                        boolean_memory_length =
                            boolean_memory_length.max(instruction.a_field() + 1);
                    }
                    Type::Byte => {
                        byte_memory_length = byte_memory_length.max(instruction.a_field() + 1);
                    }
                    Type::Character => {
                        character_memory_length =
                            character_memory_length.max(instruction.a_field() + 1);
                    }
                    Type::Float => {
                        float_memory_length = float_memory_length.max(instruction.a_field() + 1);
                    }
                    Type::Integer => {
                        integer_memory_length =
                            integer_memory_length.max(instruction.a_field() + 1);
                    }
                    Type::String => {
                        string_memory_length = string_memory_length.max(instruction.a_field() + 1);
                    }
                    Type::List(_) => {
                        list_memory_length = list_memory_length.max(instruction.a_field() + 1);
                    }
                    Type::Function(_) => {
                        function_memory_length =
                            function_memory_length.max(instruction.a_field() + 1);
                    }
                    _ => {}
                }
            }

            instructions.push(instruction);
            positions.push(position);
        }

        Chunk {
            name: self.mode.into_name().map(DustString::from),
            r#type: self.r#type,
            instructions,
            positions,
            character_constants: self.character_constants,
            float_constants: self.float_constants,
            integer_constants: self.integer_constants,
            string_constants: self.string_constants,
            locals: self.locals,
            prototypes: self.prototypes,
            arguments: self
                .arguments
                .into_iter()
                .map(|(values, _)| values)
                .collect(),
            boolean_memory_length,
            byte_memory_length,
            character_memory_length,
            float_memory_length,
            integer_memory_length,
            string_memory_length,
            list_memory_length,
            function_memory_length,
            prototype_index: self.prototype_index,
        }
    }

    fn optimize_instructions(&mut self) {
        let logging = span!(Level::TRACE, "Optimize");
        let _enter = logging.enter();

        let mut boolean_address_rankings = Vec::<(usize, Address)>::new();
        let mut byte_address_rankings = Vec::<(usize, Address)>::new();
        let mut character_address_rankings = Vec::<(usize, Address)>::new();
        let mut float_address_rankings = Vec::<(usize, Address)>::new();
        let mut integer_address_rankings = Vec::<(usize, Address)>::new();
        let mut string_address_rankings = Vec::<(usize, Address)>::new();
        let mut list_address_rankings = Vec::<(usize, Address)>::new();
        let mut function_address_rankings = Vec::<(usize, Address)>::new();
        let mut disqualified = HashSet::<Address>::new();

        // Increases the rank of the given address by 1 to indicate that it was used. If the
        // address has no existing rank, it is inserted in sorted order with a rank of 0.
        let mut increment_rank = |address: Address, r#type: OperandType, increment: usize| {
            if matches!(address.memory, MemoryKind::CELL | MemoryKind::CONSTANT) {
                return;
            }

            let address_rankings = match r#type {
                OperandType::BOOLEAN => &mut boolean_address_rankings,
                OperandType::BYTE => &mut byte_address_rankings,
                OperandType::CHARACTER => &mut character_address_rankings,
                OperandType::FLOAT => &mut float_address_rankings,
                OperandType::INTEGER => &mut integer_address_rankings,
                OperandType::STRING
                | OperandType::CHARACTER_STRING
                | OperandType::STRING_CHARACTER => &mut string_address_rankings,
                OperandType::LIST => &mut list_address_rankings,
                OperandType::FUNCTION if address.is_heap() => &mut function_address_rankings,
                _ => return,
            };

            match address_rankings.binary_search_by_key(&address, |(_, address)| *address) {
                Ok(index) => {
                    let rank = &mut address_rankings[index].0;
                    *rank = rank.saturating_add(increment);
                }
                Err(index) => {
                    address_rankings.insert(index, (increment, address));
                }
            }
        };
        let mut disqualify = |address: Address| {
            if matches!(address.memory, MemoryKind::CELL | MemoryKind::CONSTANT) {
                return;
            }

            disqualified.insert(address);
        };

        for (instruction, _, _) in &self.instructions {
            let destination_address = instruction.destination();
            let b_address = instruction.b_address();
            let c_address = instruction.c_address();
            let r#type = instruction.operand_type();

            match instruction.operation() {
                Operation::CLOSE => {
                    for index in b_address.index..=c_address.index {
                        let address = Address::new(index, b_address.memory);

                        disqualify(address);
                    }
                }
                Operation::LOAD => {
                    increment_rank(destination_address, r#type, 1);
                    increment_rank(b_address, r#type, 1);
                }
                Operation::CALL_NATIVE => {
                    increment_rank(destination_address, r#type, 2);
                }
                Operation::ADD
                | Operation::SUBTRACT
                | Operation::MULTIPLY
                | Operation::DIVIDE
                | Operation::MODULO => {
                    increment_rank(destination_address, r#type, 2);
                    increment_rank(b_address, r#type, 2);
                    increment_rank(c_address, r#type, 2);
                }
                Operation::EQUAL | Operation::LESS | Operation::LESS_EQUAL => {
                    increment_rank(b_address, r#type, 2);
                    increment_rank(c_address, r#type, 2);
                }
                Operation::TEST | Operation::RETURN => {
                    increment_rank(b_address, r#type, 2);
                }
                Operation::NEGATE | Operation::CALL => {
                    increment_rank(destination_address, r#type, 2);
                    increment_rank(b_address, r#type, 2);
                }
                _ => {}
            }
        }

        for (values, _) in &self.arguments {
            for (address, r#type) in values {
                let operand_type = match r#type {
                    TypeKind::Boolean => OperandType::BOOLEAN,
                    TypeKind::Byte => OperandType::BYTE,
                    TypeKind::Character => OperandType::CHARACTER,
                    TypeKind::Float => OperandType::FLOAT,
                    TypeKind::Integer => OperandType::INTEGER,
                    TypeKind::String => OperandType::STRING,
                    TypeKind::List => OperandType::LIST,
                    TypeKind::Function => OperandType::FUNCTION,
                    _ => continue,
                };

                increment_rank(*address, operand_type, 2);
            }
        }

        // A map in which the keys are addresses that need to be replaced and the values are their
        // intended replacements.
        let mut replacements = HashMap::new();
        let get_top_ranks_with_registers =
            |address_rankings: Vec<(usize, Address)>, r#type: TypeKind| {
                address_rankings
                    .into_iter()
                    .zip(repeat(r#type))
                    .filter_map(|((rank, address), r#type)| {
                        if !disqualified.contains(&address) {
                            Some((rank, address, r#type.clone()))
                        } else {
                            None
                        }
                    })
                    .take(REGISTER_COUNT)
                    .zip(0..)
            };

        for ((rank, old_address, r#type), register_index) in
            get_top_ranks_with_registers(boolean_address_rankings, TypeKind::Boolean)
                .chain(get_top_ranks_with_registers(
                    byte_address_rankings,
                    TypeKind::Byte,
                ))
                .chain(get_top_ranks_with_registers(
                    character_address_rankings,
                    TypeKind::Character,
                ))
                .chain(get_top_ranks_with_registers(
                    float_address_rankings,
                    TypeKind::Float,
                ))
                .chain(get_top_ranks_with_registers(
                    integer_address_rankings,
                    TypeKind::Integer,
                ))
                .chain(get_top_ranks_with_registers(
                    string_address_rankings,
                    TypeKind::String,
                ))
                .chain(get_top_ranks_with_registers(
                    list_address_rankings,
                    TypeKind::List,
                ))
                .chain(get_top_ranks_with_registers(
                    function_address_rankings,
                    TypeKind::Function,
                ))
        {
            let new_address = match r#type {
                TypeKind::Boolean => Address::stack(register_index),
                TypeKind::Byte => Address::stack(register_index),
                TypeKind::Character => Address::stack(old_address.index),
                TypeKind::Float => Address::stack(register_index),
                TypeKind::Integer => Address::stack(register_index),
                TypeKind::String => Address::stack(register_index),
                TypeKind::List => Address::stack(register_index),
                TypeKind::Function => Address::stack(register_index),
                _ => todo!(),
            };

            trace!(
                "{old_address} -> {new_address} Usage Rank: {rank}",
                old_address = old_address.to_string(r#type),
                new_address = new_address.to_string(r#type),
            );

            replacements.insert(old_address, new_address);
        }

        trace!(
            "{} addresses disqualified for register optimization",
            disqualified.len()
        );

        for (instruction, _, _) in &mut self.instructions {
            let destination = instruction.destination();
            let b_address = instruction.b_address();
            let c_address = instruction.c_address();

            match instruction.operation() {
                Operation::NEGATE | Operation::CALL => {
                    if let Some(replacement) = replacements.get(&destination) {
                        instruction.set_destination(*replacement);
                    }

                    if let Some(replacement) = replacements.get(&b_address) {
                        instruction.set_b_address(*replacement);
                    }
                }
                Operation::LOAD | Operation::CALL_NATIVE => {
                    if let Some(replacement) = replacements.get(&destination) {
                        instruction.set_destination(*replacement);
                    }
                }
                Operation::ADD
                | Operation::SUBTRACT
                | Operation::MULTIPLY
                | Operation::DIVIDE
                | Operation::MODULO => {
                    if let Some(replacement) = replacements.get(&destination) {
                        instruction.set_destination(*replacement);
                    }

                    if let Some(replacement) = replacements.get(&b_address) {
                        instruction.set_b_address(*replacement);
                    }

                    if let Some(replacement) = replacements.get(&c_address) {
                        instruction.set_c_address(*replacement);
                    }
                }
                Operation::EQUAL | Operation::LESS | Operation::LESS_EQUAL => {
                    if let Some(replacement) = replacements.get(&b_address) {
                        instruction.set_b_address(*replacement);
                    }

                    if let Some(replacement) = replacements.get(&c_address) {
                        instruction.set_c_address(*replacement);
                    }
                }
                Operation::TEST | Operation::RETURN => {
                    if let Some(replacement) = replacements.get(&b_address) {
                        instruction.set_b_address(*replacement);
                    }
                }
                _ => {}
            }
        }

        for local in &mut self.locals {
            if let Some(replacement) = replacements.get(&local.address) {
                local.address = *replacement;
            }
        }

        for arguments in &mut self.arguments {
            for (address, _) in &mut arguments.0 {
                if let Some(replacement) = replacements.get(address) {
                    *address = *replacement;
                }
            }
        }
    }

    fn is_eof(&self) -> bool {
        matches!(self.current_token, Token::Eof)
    }

    fn next_boolean_heap_index(&mut self) -> u16 {
        let reclaimable_index = self
            .reclaimable_memory
            .iter()
            .position(|(_, r#type)| *r#type == TypeKind::Boolean);

        if let Some(index) = reclaimable_index {
            trace!("Reclaiming boolean memory at index {index}");

            let (address, _) = self.reclaimable_memory.remove(index);

            return address.index;
        }

        self.instructions.iter().fold(
            self.minimum_boolean_memory_index,
            |acc, (instruction, r#type, _)| {
                if instruction.yields_value()
                    && r#type == &Type::Boolean
                    && instruction.a_field() >= acc
                {
                    instruction.a_field() + 1
                } else {
                    acc
                }
            },
        )
    }

    fn next_byte_heap_index(&mut self) -> u16 {
        let reclaimable_index = self
            .reclaimable_memory
            .iter()
            .position(|(_, r#type)| *r#type == TypeKind::Byte);

        if let Some(index) = reclaimable_index {
            trace!("Reclaiming byte memory at index {index}");

            let (address, _) = self.reclaimable_memory.remove(index);

            return address.index;
        }

        self.instructions.iter().fold(
            self.minimum_byte_memory_index,
            |acc, (instruction, r#type, _)| {
                if instruction.yields_value()
                    && r#type == &Type::Byte
                    && instruction.a_field() >= acc
                {
                    instruction.a_field() + 1
                } else {
                    acc
                }
            },
        )
    }

    fn next_character_heap_index(&mut self) -> u16 {
        let reclaimable_index = self
            .reclaimable_memory
            .iter()
            .position(|(_, r#type)| *r#type == TypeKind::Character);

        if let Some(index) = reclaimable_index {
            trace!("Reclaiming character memory at index {index}");

            let (address, _) = self.reclaimable_memory.remove(index);

            return address.index;
        }

        self.instructions.iter().fold(
            self.minimum_character_memory_index,
            |acc, (instruction, r#type, _)| {
                if instruction.yields_value()
                    && r#type == &Type::Character
                    && instruction.a_field() >= acc
                {
                    instruction.a_field() + 1
                } else {
                    acc
                }
            },
        )
    }

    fn next_float_heap_index(&mut self) -> u16 {
        let reclaimable_index = self
            .reclaimable_memory
            .iter()
            .position(|(_, r#type)| *r#type == TypeKind::Float);

        if let Some(index) = reclaimable_index {
            trace!("Reclaiming float memory at index {index}");

            let (address, _) = self.reclaimable_memory.remove(index);

            return address.index;
        }

        self.instructions.iter().fold(
            self.minimum_float_memory_index,
            |acc, (instruction, r#type, _)| {
                if instruction.yields_value()
                    && r#type == &Type::Float
                    && instruction.a_field() >= acc
                {
                    instruction.a_field() + 1
                } else {
                    acc
                }
            },
        )
    }

    fn next_integer_heap_index(&mut self) -> u16 {
        let reclaimable_index = self
            .reclaimable_memory
            .iter()
            .position(|(_, r#type)| *r#type == TypeKind::Integer);

        if let Some(index) = reclaimable_index {
            trace!("Reclaiming integer memory at index {index}");

            let (address, _) = self.reclaimable_memory.remove(index);

            return address.index;
        }

        self.instructions.iter().fold(
            self.minimum_integer_memory_index,
            |acc, (instruction, r#type, _)| {
                if instruction.yields_value()
                    && r#type == &Type::Integer
                    && instruction.a_field() >= acc
                {
                    instruction.a_field() + 1
                } else {
                    acc
                }
            },
        )
    }

    fn next_string_heap_index(&mut self) -> u16 {
        let reclaimable_index = self
            .reclaimable_memory
            .iter()
            .position(|(_, r#type)| *r#type == TypeKind::String);

        if let Some(index) = reclaimable_index {
            trace!("Reclaiming string memory at index {index}");

            let (address, _) = self.reclaimable_memory.remove(index);

            return address.index;
        }

        self.instructions.iter().fold(
            self.minimum_string_memory_index,
            |acc, (instruction, r#type, _)| {
                if instruction.yields_value()
                    && r#type == &Type::String
                    && instruction.a_field() >= acc
                {
                    instruction.a_field() + 1
                } else {
                    acc
                }
            },
        )
    }

    fn next_list_heap_index(&mut self) -> u16 {
        let reclaimable_index = self
            .reclaimable_memory
            .iter()
            .position(|(_, r#type)| *r#type == TypeKind::List);

        if let Some(index) = reclaimable_index {
            trace!("Reclaiming list memory at index {index}");

            let (address, _) = self.reclaimable_memory.remove(index);

            return address.index;
        }

        self.instructions.iter().fold(
            self.minimum_list_memory_index,
            |acc, (instruction, r#type, _)| {
                if instruction.yields_value()
                    && matches!(r#type, Type::List(_))
                    && instruction.a_field() >= acc
                {
                    instruction.a_field() + 1
                } else {
                    acc
                }
            },
        )
    }

    fn next_function_heap_index(&mut self) -> u16 {
        let reclaimable_index = self
            .reclaimable_memory
            .iter()
            .position(|(_, r#type)| *r#type == TypeKind::Function);

        if let Some(index) = reclaimable_index {
            trace!("Reclaiming function memory at index {index}");

            let (address, _) = self.reclaimable_memory.remove(index);

            return address.index;
        }

        self.instructions.iter().fold(
            self.minimum_function_memory_index,
            |acc, (instruction, r#type, _)| {
                if instruction.yields_value()
                    && matches!(r#type, Type::Function(_))
                    && instruction.a_field() >= acc
                {
                    instruction.a_field() + 1
                } else {
                    acc
                }
            },
        )
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
    fn get_local(&self, identifier: &str) -> Option<&Local> {
        self.locals.iter().find(|local| {
            self.string_constants
                .get(local.identifier_index as usize)
                .is_some_and(|string| string == identifier)
        })
    }

    /// Adds a new local to `self.locals` and returns a tuple holding the index of the new local and
    /// the index of its identifier in `self.string_constants`.
    fn declare_local(
        &mut self,
        identifier: &str,
        address: Address,
        r#type: Type,
        is_mutable: bool,
        scope: BlockScope,
    ) -> (u16, u16) {
        info!("Declaring local {identifier}");

        let identifier = DustString::from(identifier);
        let identifier_index = self.push_or_get_constant_string(identifier);
        let local_index = self.locals.len() as u16;

        match r#type.kind() {
            TypeKind::Boolean => self.minimum_boolean_memory_index += 1,
            TypeKind::Byte => self.minimum_byte_memory_index += 1,
            TypeKind::Character => self.minimum_character_memory_index += 1,
            TypeKind::Float => self.minimum_float_memory_index += 1,
            TypeKind::Integer => self.minimum_integer_memory_index += 1,
            TypeKind::String => self.minimum_string_memory_index += 1,
            TypeKind::List => self.minimum_list_memory_index += 1,
            TypeKind::Function => self.minimum_function_memory_index += 1,
            _ => todo!(),
        }

        self.locals.push(Local::new(
            identifier_index,
            address,
            r#type,
            is_mutable,
            scope,
        ));

        (local_index, identifier_index)
    }

    fn declare_global(&mut self, identifier: &str, r#type: Type, is_mutable: bool) -> u16 {
        info!("Declaring global {identifier}");

        let identifier = DustString::from(identifier);
        let cell_index = self.globals.len() as u16;

        self.globals
            .insert(identifier, Global::new(cell_index, r#type, is_mutable));

        cell_index
    }

    fn push_or_get_constant_character(&mut self, character: char) -> u16 {
        if let Some(index) = self
            .character_constants
            .iter()
            .position(|constant| constant == &character)
        {
            index as u16
        } else {
            let index = self.character_constants.len() as u16;

            self.character_constants.push(character);

            index
        }
    }

    fn push_or_get_constant_float(&mut self, float: f64) -> u16 {
        if let Some(index) = self
            .float_constants
            .iter()
            .position(|constant| constant == &float)
        {
            index as u16
        } else {
            let index = self.float_constants.len() as u16;

            self.float_constants.push(float);

            index
        }
    }

    fn push_or_get_constant_integer(&mut self, integer: i64) -> u16 {
        if let Some(index) = self
            .integer_constants
            .iter()
            .position(|constant| constant == &integer)
        {
            index as u16
        } else {
            let index = self.integer_constants.len() as u16;

            self.integer_constants.push(integer);

            index
        }
    }

    fn push_or_get_constant_string(&mut self, string: DustString) -> u16 {
        if let Some(index) = self
            .string_constants
            .iter()
            .position(|constant| constant == &string)
        {
            index as u16
        } else {
            let index = self.string_constants.len() as u16;

            self.string_constants.push(string);

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
            let destination = Address::heap(self.next_boolean_heap_index());
            let operand = Address::new(boolean as u16, MemoryKind::default());
            let load = Instruction::load(destination, operand, OperandType::BOOLEAN, false);

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
        text.parse::<bool>().unwrap()
    }

    fn parse_byte(&mut self) -> Result<(), CompileError> {
        let position = self.current_position;

        if let Token::Byte(text) = self.current_token {
            self.advance()?;

            let byte = self.parse_byte_value(text)?;
            let destination = Address::heap(self.next_byte_heap_index());
            let operand = Address::new(byte as u16, MemoryKind::default());
            let load_encoded = Instruction::load(destination, operand, OperandType::BYTE, false);

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

            let destination = self.next_character_heap_index();
            let constant_index = if let Some(index) = self
                .character_constants
                .iter()
                .position(|constant| constant == &character)
            {
                index as u16
            } else {
                let index = self.character_constants.len() as u16;

                self.character_constants.push(character);

                index
            };
            let load_constant = Instruction::load(
                Address::heap(destination),
                Address::constant(constant_index),
                OperandType::CHARACTER,
                false,
            );

            self.emit_instruction(load_constant, Type::Character, position);

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

            let float = self.parse_float_value(text)?;
            let destination = self.next_float_heap_index();
            let constant_index = if let Some(index) = self
                .float_constants
                .iter()
                .position(|constant| constant == &float)
            {
                index as u16
            } else {
                let index = self.float_constants.len() as u16;

                self.float_constants.push(float);

                index
            };
            let load_constant = Instruction::load(
                Address::heap(destination),
                Address::constant(constant_index),
                OperandType::FLOAT,
                false,
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

            let integer = self.parse_integer_value(text);
            let constant_index = if let Some(index) = self
                .integer_constants
                .iter()
                .position(|constant| constant == &integer)
            {
                index as u16
            } else {
                let index = self.integer_constants.len() as u16;

                self.integer_constants.push(integer);

                index
            };
            let destination = self.next_integer_heap_index();
            let load_constant = Instruction::load(
                Address::heap(destination),
                Address::constant(constant_index),
                OperandType::INTEGER,
                false,
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

            let string = DustString::from(text);
            let constant_index = if let Some(index) = self
                .string_constants
                .iter()
                .position(|constant| constant == &string)
            {
                index as u16
            } else {
                let index = self.string_constants.len() as u16;

                self.string_constants.push(string);

                index
            };
            let destination = self.next_string_heap_index();
            let load_constant = Instruction::load(
                Address::heap(destination),
                Address::constant(constant_index),
                OperandType::STRING,
                false,
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
        let (address, push_back) = self.handle_binary_argument(&previous_instruction);

        if push_back {
            self.instructions.push((
                previous_instruction,
                previous_type.clone(),
                previous_position,
            ))
        }

        let destination_index = match previous_type {
            Type::Boolean => self.next_boolean_heap_index(),
            Type::Float => self.next_float_heap_index(),
            Type::Integer => self.next_integer_heap_index(),
            _ => match operator {
                Token::Minus => {
                    return Err(CompileError::CannotNegateType {
                        argument_type: previous_type,
                        position: previous_position,
                    });
                }
                Token::Bang => {
                    return Err(CompileError::CannotNotType {
                        argument_type: previous_type,
                        position: previous_position,
                    });
                }
                _ => {
                    return Err(CompileError::ExpectedTokenMultiple {
                        expected: &[TokenKind::Bang, TokenKind::Minus],
                        found: operator.to_owned(),
                        position: operator_position,
                    });
                }
            },
        };
        let destination = Address::heap(destination_index);
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
            _ => unreachable!(
                "If used correctly, the pratt parsing algorithm should make this impossible."
            ),
        };

        self.emit_instruction(instruction, previous_type, operator_position);

        Ok(())
    }

    /// Takes an instruction and returns an [`address`] that corresponds to its address and a
    /// boolean indicating whether the instruction should be pushed back onto the instruction list.
    /// If `false`, the address makes the instruction irrelevant.
    fn handle_binary_argument(&mut self, instruction: &Instruction) -> (Address, bool) {
        let address = instruction.destination();
        let push_back = match instruction.operation() {
            Operation::LOAD
            | Operation::CALL
            | Operation::CALL_NATIVE
            | Operation::ADD
            | Operation::SUBTRACT
            | Operation::MULTIPLY
            | Operation::DIVIDE
            | Operation::MODULO
            | Operation::NEGATE => true,
            _ => !instruction.yields_value(),
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
        let (left, push_back_left) = self.handle_binary_argument(&left_instruction);

        let left_is_mutable_variable = match left_instruction.operation() {
            Operation::LOAD => {
                let Load { operand, .. } = Load::from(&left_instruction);

                if operand.memory == MemoryKind::HEAP {
                    self.locals
                        .iter()
                        .find_map(|local| {
                            if local.address == operand {
                                Some(local.is_mutable)
                            } else {
                                None
                            }
                        })
                        .unwrap_or(false)
                } else if operand.memory == MemoryKind::CELL {
                    self.globals
                        .values()
                        .find_map(|global| {
                            if global.cell_index == operand.index {
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
        let rule = ParseRule::<REGISTER_COUNT>::from(&operator);
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
        let (right, push_back_right) = self.handle_binary_argument(&right_instruction);

        check_math_type(&right_type, operator, &right_position)?;
        check_math_types(
            &left_type,
            &left_position,
            operator,
            &right_type,
            &right_position,
        )?;
        let right_type_kind = right_type.kind();

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
        let destination_index = if is_assignment {
            left.index
        } else {
            match left_type {
                Type::Byte => self.next_byte_heap_index(),
                Type::Character => self.next_string_heap_index(),
                Type::Float => self.next_float_heap_index(),
                Type::Integer => self.next_integer_heap_index(),
                Type::String => self.next_string_heap_index(),
                _ => unreachable!(),
            }
        };
        let destination = Address::heap(destination_index);
        let operand_type = match left_type {
            Type::Byte => OperandType::BYTE,
            Type::Character => match right_type_kind {
                TypeKind::Character => OperandType::CHARACTER,
                TypeKind::String => OperandType::CHARACTER_STRING,
                _ => unreachable!(),
            },
            Type::Float => OperandType::FLOAT,
            Type::Integer => OperandType::INTEGER,
            Type::String => match right_type_kind {
                TypeKind::Character => OperandType::CHARACTER_STRING,
                TypeKind::String => OperandType::STRING,
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

        // TODO: Check if the left type is a valid type for comparison

        let (left, push_back_left) = self.handle_binary_argument(&left_instruction);

        if push_back_left {
            self.instructions
                .push((left_instruction, left_type, left_position));
        }

        let operator = self.current_token;
        let operator_position = self.current_position;
        let rule = ParseRule::<REGISTER_COUNT>::from(&operator);

        self.advance()?;
        self.parse_sub_expression(&rule.precedence)?;

        let (right_instruction, right_type, right_position) =
            self.instructions
                .pop()
                .ok_or_else(|| CompileError::ExpectedExpression {
                    found: self.previous_token.to_owned(),
                    position: self.previous_position,
                })?;

        // TODO: Check if the right type is a valid type for comparison

        let (right, push_back_right) = self.handle_binary_argument(&right_instruction);
        let right_type_kind = right_type.kind();

        if push_back_right {
            self.instructions
                .push((right_instruction, right_type, right_position));
        }

        // TODO: Check if the left and right types are compatible

        let operand_type = match right_type_kind {
            TypeKind::Boolean => OperandType::BOOLEAN,
            TypeKind::Byte => OperandType::BYTE,
            TypeKind::Character => OperandType::CHARACTER,
            TypeKind::Float => OperandType::FLOAT,
            TypeKind::Integer => OperandType::INTEGER,
            TypeKind::String => OperandType::STRING,
            TypeKind::List => OperandType::LIST,
            TypeKind::Function => OperandType::FUNCTION,
            _ => todo!(),
        };
        let comparison = match operator {
            Token::DoubleEqual => Instruction::equal(true, left, right, operand_type),
            Token::BangEqual => Instruction::equal(false, left, right, operand_type),
            Token::Less => Instruction::less(true, left, right, operand_type),
            Token::LessEqual => Instruction::less_equal(true, left, right, operand_type),
            Token::Greater => Instruction::less_equal(false, left, right, operand_type),
            Token::GreaterEqual => Instruction::less(false, left, right, operand_type),
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
        let jump = Instruction::jump(1, true);
        let destination_index = self.next_boolean_heap_index();
        let destination = Address::heap(destination_index);
        let true_as_address = Address::new(true as u16, MemoryKind::default());
        let load_true = Instruction::load(destination, true_as_address, OperandType::BOOLEAN, true);
        let false_as_address = Address::new(false as u16, MemoryKind::default());
        let load_false =
            Instruction::load(destination, false_as_address, OperandType::BOOLEAN, false);
        let comparison_position = Span(left_position.0, right_position.1);

        self.emit_instruction(comparison, Type::Boolean, comparison_position);
        self.emit_instruction(jump, Type::None, comparison_position);
        self.emit_instruction(load_true, Type::Boolean, comparison_position);
        self.emit_instruction(load_false, Type::Boolean, comparison_position);

        Ok(())
    }

    fn parse_logical_binary(&mut self) -> Result<(), CompileError> {
        let is_logic_chain = matches!(
            self.previous_token,
            Token::DoubleAmpersand | Token::DoublePipe | Token::Bang
        );
        let (left_instruction, left_type, left_position) =
            self.instructions
                .pop()
                .ok_or_else(|| CompileError::ExpectedExpression {
                    found: self.previous_token.to_owned(),
                    position: self.previous_position,
                })?;

        if left_type != Type::Boolean {
            return Err(CompileError::ExpectedBoolean {
                found: self.previous_token.to_owned(),
                position: left_position,
            });
        }

        let (left, push_back_left) = self.handle_binary_argument(&left_instruction);

        if push_back_left {
            self.instructions
                .push((left_instruction, left_type.clone(), left_position));
        }

        let operator = self.current_token;
        let operator_position = self.current_position;
        let rule = ParseRule::<REGISTER_COUNT>::from(&operator);
        let test_boolean = match operator {
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
        let test = Instruction::test(left, test_boolean);

        self.emit_instruction(test, Type::None, operator_position);

        let jump_index = self.instructions.len();
        let jump_position = self.current_position;

        self.advance()?;
        self.parse_sub_expression(&rule.precedence)?;

        let right_type = &self.instructions.last().unwrap().1;

        if right_type != &Type::Boolean {
            return Err(CompileError::ExpectedBoolean {
                found: self.previous_token.to_owned(),
                position: left_position,
            });
        }

        let instruction_count = self.instructions.len();

        if instruction_count == jump_index + 1 {
            let (last_instruction, _, _) = self.instructions.last_mut().unwrap();

            if last_instruction.yields_value() {
                last_instruction.set_a_field(left.index);
            }
        } else if matches!(
            self.get_last_operations(),
            Some([
                Operation::EQUAL | Operation::LESS | Operation::LESS_EQUAL,
                Operation::JUMP,
                Operation::LOAD,
                Operation::LOAD,
            ])
        ) {
            let load_instructions = if cfg!(debug_assertions) {
                self.instructions
                    .get_disjoint_mut([instruction_count - 1, instruction_count - 2])
                    .unwrap() // Safe because the indices in bounds and do not overlap
            } else {
                unsafe {
                    self.instructions
                        .get_disjoint_unchecked_mut([instruction_count - 1, instruction_count - 2])
                }
            };

            let Load {
                operand,
                r#type,
                jump_next,
                ..
            } = Load::from(&load_instructions[0].0);
            load_instructions[0].0 =
                Instruction::load(Address::heap(left.index), operand, r#type, jump_next);

            let Load {
                operand,
                r#type,
                jump_next,
                ..
            } = Load::from(&load_instructions[1].0);
            load_instructions[1].0 =
                Instruction::load(Address::heap(left.index), operand, r#type, jump_next);
        }

        let instructions_length = self.instructions.len();
        let jump_distance = if is_logic_chain
            || !matches!(
                self.current_token,
                Token::DoubleAmpersand | Token::DoublePipe | Token::Bang
            ) {
            instructions_length - jump_index
        } else {
            instructions_length - jump_index + 1
        } as u16;
        let jump = Instruction::jump(jump_distance, true);

        self.instructions
            .insert(jump_index, (jump, Type::None, jump_position));

        Ok(())
    }

    fn parse_variable(&mut self) -> Result<(), CompileError> {
        let start_position = self.current_position;
        let identifier = if let Token::Identifier(text) = self.current_token {
            self.advance()?;

            text
        } else {
            return Err(CompileError::ExpectedToken {
                expected: TokenKind::Identifier,
                found: self.current_token.to_owned(),
                position: start_position,
            });
        };
        let variable_path =
            Path::new_borrowed(identifier).ok_or_else(|| CompileError::InvalidPath {
                found: identifier.to_string(),
                position: start_position,
            })?;

        let (variable_address, variable_type, is_mutable) = {
            if let Some(local) = self.get_local(identifier) {
                if !self.current_block_scope.contains(&local.scope) {
                    return Err(CompileError::VariableOutOfScope {
                        identifier: identifier.to_string(),
                        position: start_position,
                        variable_scope: local.scope,
                        access_scope: self.current_block_scope,
                    });
                }

                (local.address, local.r#type.clone(), local.is_mutable)
            } else if let Some(global) = self.globals.get(identifier).cloned() {
                let Global {
                    cell_index,
                    r#type,
                    is_mutable,
                } = global;

                (Address::cell(cell_index), r#type, is_mutable)
            } else if let Some((item, item_position)) =
                find_item!(variable_path, &*self.main_module)
            {
                let (local_address, item_type) =
                    self.bring_item_into_local_scope(identifier, item, item_position);

                (local_address, item_type, false)
            } else if let CompileMode::Function { name } = &self.mode {
                if name.as_deref() == Some(identifier) {
                    let destination_index = self.next_function_heap_index();
                    let destination = Address::heap(destination_index);
                    let load_self = Instruction::load(
                        destination,
                        Address::default(),
                        OperandType::FUNCTION_SELF,
                        false,
                    );

                    self.emit_instruction(load_self, Type::FunctionSelf, start_position);

                    return Ok(());
                } else if self.allow_native_functions {
                    if let Some(native_function) = NativeFunction::from_str(identifier) {
                        return self.parse_call_native(native_function, start_position);
                    }
                }

                return Err(CompileError::UndeclaredVariable {
                    identifier: identifier.to_string(),
                    position: start_position,
                });
            } else {
                if self.allow_native_functions {
                    if let Some(native_function) = NativeFunction::from_str(identifier) {
                        return self.parse_call_native(native_function, start_position);
                    }
                }

                return Err(CompileError::UndeclaredVariable {
                    identifier: identifier.to_string(),
                    position: start_position,
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
                    position: start_position,
                });
            }
        }

        let (destination_index, operand_type) = match variable_type {
            Type::Boolean => (self.next_boolean_heap_index(), OperandType::BOOLEAN),
            Type::Byte => (self.next_byte_heap_index(), OperandType::BYTE),
            Type::Character => (self.next_character_heap_index(), OperandType::CHARACTER),
            Type::Float => (self.next_float_heap_index(), OperandType::FLOAT),
            Type::Integer => (self.next_integer_heap_index(), OperandType::INTEGER),
            Type::String => (self.next_string_heap_index(), OperandType::STRING),
            Type::List(_) => (self.next_list_heap_index(), OperandType::LIST),
            Type::Function(_) => (self.next_function_heap_index(), OperandType::FUNCTION),
            Type::FunctionSelf => (self.next_function_heap_index(), OperandType::FUNCTION_SELF),
            _ => todo!(),
        };
        let destination = Address::heap(destination_index);
        let load = Instruction::load(destination, variable_address, operand_type, false);

        self.emit_instruction(load, variable_type, self.previous_position);

        Ok(())
    }

    fn parse_type(&mut self) -> Result<Type, CompileError> {
        match self.current_token {
            Token::Any => {
                self.advance()?;

                Ok(Type::Any)
            }
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
                    TokenKind::Any,
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
        let start_boolean_memory_index = self.next_boolean_heap_index();
        let start_byte_memory_index = self.next_byte_heap_index();
        let start_character_memory_index = self.next_character_heap_index();
        let start_float_memory_index = self.next_float_heap_index();
        let start_integer_memory_index = self.next_integer_heap_index();
        let start_string_memory_index = self.next_string_heap_index();
        let start_list_memory_index = self.next_list_heap_index();
        let start_function_memory_index = self.next_function_heap_index();

        self.block_index += 1;
        self.current_block_scope.begin(self.block_index);

        while !self.allow(Token::RightBrace)? && !self.is_eof() {
            self.parse(Precedence::None)?;
        }

        let end_boolean_memory_index = self.next_boolean_heap_index();
        let end_byte_memory_index = self.next_byte_heap_index();
        let end_character_memory_index = self.next_character_heap_index();
        let end_float_memory_index = self.next_float_heap_index();
        let end_integer_memory_index = self.next_integer_heap_index();
        let end_string_memory_index = self.next_string_heap_index();
        let end_list_memory_index = self.next_list_heap_index();
        let end_function_memory_index = self.next_function_heap_index();

        for (start, end, r#type) in [
            (
                start_boolean_memory_index,
                end_boolean_memory_index,
                TypeKind::Boolean,
            ),
            (
                start_byte_memory_index,
                end_byte_memory_index,
                TypeKind::Byte,
            ),
            (
                start_character_memory_index,
                end_character_memory_index,
                TypeKind::Character,
            ),
            (
                start_float_memory_index,
                end_float_memory_index,
                TypeKind::Float,
            ),
            (
                start_integer_memory_index,
                end_integer_memory_index,
                TypeKind::Integer,
            ),
            (
                start_string_memory_index,
                end_string_memory_index,
                TypeKind::String,
            ),
            (
                start_list_memory_index,
                end_list_memory_index,
                TypeKind::List,
            ),
            (
                start_function_memory_index,
                end_function_memory_index,
                TypeKind::Function,
            ),
        ] {
            for i in start..end {
                let address = Address::heap(i);

                if !self
                    .locals
                    .iter()
                    .any(|local| local.address == address && local.scope == starting_scope)
                {
                    self.reclaimable_memory.push((address, r#type));
                }
            }
        }

        self.current_block_scope.end(starting_block);

        Ok(())
    }

    fn parse_list(&mut self) -> Result<(), CompileError> {
        let start = self.current_position.0;

        self.advance()?;

        let mut item_type = Type::None;
        let mut start_register = None;

        while !self.allow(Token::RightBracket)? {
            let start_boolean_address = self.next_boolean_heap_index();
            let start_byte_address = self.next_byte_heap_index();
            let start_character_address = self.next_character_heap_index();
            let start_float_address = self.next_float_heap_index();
            let start_integer_address = self.next_integer_heap_index();
            let start_string_address = self.next_string_heap_index();
            let start_list_address = self.next_list_heap_index();
            let start_function_address = self.next_function_heap_index();

            self.parse_expression()?;
            self.allow(Token::Comma)?;

            if item_type == Type::None {
                item_type = self.get_last_instruction_type();
            } else {
                // TODO: Check if the item type the same as the previous item type
            }

            if start_register.is_none() {
                let first_index = self.instructions.last().unwrap().0.a_field();

                start_register = Some(first_index);
            }

            let end_boolean_address = self.next_boolean_heap_index();
            let end_byte_address = self.next_byte_heap_index();
            let end_character_address = self.next_character_heap_index();
            let end_float_address = self.next_float_heap_index();
            let end_integer_address = self.next_integer_heap_index();
            let end_string_address = self.next_string_heap_index();
            let end_list_address = self.next_list_heap_index();
            let end_function_address = self.next_function_heap_index();

            for (start, end, r#type) in [
                (
                    start_boolean_address,
                    end_boolean_address,
                    OperandType::BOOLEAN,
                ),
                (start_byte_address, end_byte_address, OperandType::BYTE),
                (
                    start_character_address,
                    end_character_address,
                    OperandType::CHARACTER,
                ),
                (start_float_address, end_float_address, OperandType::FLOAT),
                (
                    start_integer_address,
                    end_integer_address,
                    OperandType::INTEGER,
                ),
                (
                    start_string_address,
                    end_string_address,
                    OperandType::STRING,
                ),
                (start_list_address, end_list_address, OperandType::LIST),
                (
                    start_function_address,
                    end_function_address,
                    OperandType::FUNCTION,
                ),
            ] {
                let used_addresses = end - start;

                if used_addresses > 1 {
                    let end_closing = end - 2;
                    let close = Instruction::close(
                        Address::heap(start),
                        Address::heap(end_closing),
                        r#type,
                    );

                    self.emit_instruction(close, Type::None, self.previous_position);
                }
            }
        }

        let end = self.previous_position.1;
        let (end_register, operand_type) = match item_type {
            Type::Boolean => (
                self.next_boolean_heap_index().saturating_sub(1),
                OperandType::BOOLEAN,
            ),
            Type::Byte => (
                self.next_byte_heap_index().saturating_sub(1),
                OperandType::BYTE,
            ),
            Type::Character => (
                self.next_character_heap_index().saturating_sub(1),
                OperandType::CHARACTER,
            ),
            Type::Float => (
                self.next_float_heap_index().saturating_sub(1),
                OperandType::FLOAT,
            ),
            Type::Integer => (
                self.next_integer_heap_index().saturating_sub(1),
                OperandType::INTEGER,
            ),
            Type::String => (
                self.next_string_heap_index().saturating_sub(1),
                OperandType::STRING,
            ),
            Type::List { .. } => (
                self.next_list_heap_index().saturating_sub(1),
                OperandType::LIST,
            ),
            Type::Function(_) => (
                self.next_function_heap_index().saturating_sub(1),
                OperandType::FUNCTION,
            ),
            _ => todo!(),
        };
        let destination_index = self.next_list_heap_index();
        let list = Instruction::list(
            Address::heap(destination_index),
            Address::heap(start_register.unwrap_or(0)),
            Address::heap(end_register),
            operand_type,
        );
        let list_length = end_register - start_register.unwrap_or(0) + 1;

        if list_length == 1 && self.get_last_operation() == Some(Operation::CLOSE) {
            self.instructions.pop();
        }

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
                Type::Boolean => self.next_boolean_heap_index() - 1,
                _ => {
                    return Err(CompileError::ExpectedBoolean {
                        found: self.previous_token.to_owned(),
                        position: self.previous_position,
                    });
                }
            };
            let test = Instruction::test(Address::heap(address_index), true);

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
                        true,
                    );
                } else {
                    if_block_distance += 1;
                    let jump = Instruction::from(Jump {
                        offset: jump_distance as u16,
                        is_positive: true,
                    });

                    self.instructions
                        .insert(if_block_end, (jump, Type::None, if_block_end_position));
                }
            }
            2.. => {
                if_block_distance += 1;
                let jump = Instruction::from(Jump {
                    offset: jump_distance as u16,
                    is_positive: true,
                });

                self.instructions
                    .insert(if_block_end, (jump, Type::None, if_block_end_position));
            }
        }

        let jump = Instruction::jump(if_block_distance as u16, true);

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
            let address_index = match self.get_last_instruction_type() {
                Type::Boolean => self.next_boolean_heap_index() - 1,
                Type::Byte => self.next_byte_heap_index() - 1,
                Type::Character => self.next_character_heap_index() - 1,
                Type::Float => self.next_float_heap_index() - 1,
                Type::Integer => self.next_integer_heap_index() - 1,
                Type::String => self.next_string_heap_index() - 1,
                _ => todo!(),
            };
            let test = Instruction::test(Address::heap(address_index), true);

            self.emit_instruction(test, Type::None, self.current_position);
        }

        let block_start = self.instructions.len();

        self.parse_block()?;

        let block_end = self.instructions.len();
        let jump_distance = (block_end - block_start + 1) as u16;
        let jump = Instruction::from(Jump {
            offset: jump_distance,
            is_positive: true,
        });

        self.instructions
            .insert(block_start, (jump, Type::None, self.current_position));

        let jump_back_distance = (block_end - expression_start + 1) as u16;
        let jump_back = Instruction::from(Jump {
            offset: jump_back_distance,
            is_positive: false,
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
                let (return_register, address_kind) = match expression_type {
                    Type::Boolean => (self.next_boolean_heap_index() - 1, OperandType::BOOLEAN),
                    Type::Byte => (self.next_byte_heap_index() - 1, OperandType::BYTE),
                    Type::Character => {
                        (self.next_character_heap_index() - 1, OperandType::CHARACTER)
                    }
                    Type::Float => (self.next_float_heap_index() - 1, OperandType::FLOAT),
                    Type::Integer => (self.next_integer_heap_index() - 1, OperandType::INTEGER),
                    Type::String => (self.next_string_heap_index() - 1, OperandType::STRING),
                    Type::List(_) => (self.next_list_heap_index() - 1, OperandType::LIST),
                    Type::Function(_) | Type::FunctionSelf => {
                        (self.next_function_heap_index() - 1, OperandType::FUNCTION)
                    }
                    _ => todo!(),
                };

                self.update_return_type(expression_type)?;

                (true, return_register, address_kind)
            };
        let end = self.current_position.1;
        let r#return = Instruction::r#return(
            should_return_value,
            Address::heap(return_register),
            operand_type,
        );

        self.emit_instruction(r#return, Type::None, Span(start, end));

        let instruction_length = self.instructions.len();

        for (index, (instruction, _, _)) in self.instructions.iter_mut().enumerate() {
            if instruction.operation() == Operation::JUMP {
                let Jump {
                    offset,
                    is_positive,
                } = Jump::from(&*instruction);
                let offset = offset as usize;

                if is_positive && offset + index == instruction_length - 1 {
                    *instruction = Instruction::jump((offset + 1) as u16, true);
                }
            }
        }

        Ok(())
    }

    fn parse_implicit_return(&mut self) -> Result<(), CompileError> {
        if matches!(self.get_last_operation(), Some(Operation::LOAD)) {
            let (load_instruction, instruction_type, position) = self.instructions.pop().unwrap();
            let Load {
                operand, r#type, ..
            } = Load::from(&load_instruction);
            let (should_return, return_address) = if r#type == OperandType::NONE {
                (false, Address::default())
            } else {
                (true, operand)
            };
            let r#return = Instruction::r#return(should_return, return_address, r#type);

            if !should_return {
                self.instructions
                    .push((load_instruction, instruction_type.clone(), position));
            }

            self.emit_instruction(r#return, instruction_type, self.current_position);
        } else if matches!(self.get_last_operation(), Some(Operation::RETURN))
            || matches!(
                self.get_last_operations(),
                Some([Operation::RETURN, Operation::JUMP])
            )
        {
            // Do nothing if the last instruction is a return or a return followed by a jump
        } else if self.allow(Token::Semicolon)? {
            let r#return = Instruction::r#return(false, Address::default(), OperandType::NONE);

            self.emit_instruction(r#return, Type::None, self.current_position);
        } else if let Some((last_instruction, last_instruction_type, _)) = self.instructions.last()
        {
            let should_return_value = last_instruction_type != &Type::None;
            let operand_type = match last_instruction_type {
                Type::Boolean => OperandType::BOOLEAN,
                Type::Byte => OperandType::BYTE,
                Type::Character => OperandType::CHARACTER,
                Type::Float => OperandType::FLOAT,
                Type::Integer => OperandType::INTEGER,
                Type::String => OperandType::STRING,
                Type::List(_) => OperandType::LIST,
                Type::Function(_) | Type::FunctionSelf => OperandType::FUNCTION,
                Type::None => OperandType::NONE,
                _ => todo!(),
            };
            let return_address = last_instruction.destination();
            let r#return = Instruction::r#return(should_return_value, return_address, operand_type);

            self.update_return_type(last_instruction_type.clone())?;
            self.emit_instruction(r#return, Type::None, self.current_position);
        }

        Ok(())
    }

    fn parse_let(&mut self) -> Result<(), CompileError> {
        self.advance()?;

        let is_mutable = self.allow(Token::Mut)?;
        let is_cell = self.allow(Token::Cell)?;
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

        let (last_instruction, last_instruction_type, last_instruction_position) = self
            .instructions
            .last_mut()
            .ok_or_else(|| CompileError::ExpectedExpression {
                found: self.previous_token.to_owned(),
                position: self.previous_position,
            })?;

        let r#type = if let Some((r#type, position)) = explicit_type_and_position {
            r#type.check(last_instruction_type).map_err(|conflict| {
                CompileError::LetStatementTypeConflict {
                    conflict,
                    expected_position: position,
                    actual_position: *last_instruction_position,
                }
            })?;

            r#type
        } else {
            last_instruction_type.clone()
        };
        let address = last_instruction.destination();
        *last_instruction_type = Type::None;

        if is_cell {
            self.declare_global(identifier, r#type, is_mutable);
        } else {
            self.declare_local(
                identifier,
                address,
                r#type,
                is_mutable,
                self.current_block_scope,
            );
        }

        Ok(())
    }

    fn parse_function(&mut self) -> Result<(), CompileError> {
        let function_start = self.current_position.0;

        self.advance()?;

        let identifier = if let Token::Identifier(text) = self.current_token {
            self.advance()?;

            Some(text)
        } else {
            None
        };

        let mut function_compiler = if self.current_token == Token::LeftParenthesis {
            let mut compiler = ChunkCompiler::<REGISTER_COUNT>::new_function(
                self.lexer,
                identifier,
                self.current_item_scope.clone(),
                self.main_module,
                self.globals,
            )?; // This will consume the parenthesis

            compiler.prototype_index = self.prototypes.len() as u16;
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

                text
            } else {
                return Err(CompileError::ExpectedToken {
                    expected: TokenKind::Identifier,
                    found: function_compiler.current_token.to_owned(),
                    position: function_compiler.current_position,
                });
            };

            function_compiler.expect(Token::Colon)?;

            let r#type = function_compiler.parse_type()?;

            let local_memory_index = match r#type {
                Type::Boolean => function_compiler.next_boolean_heap_index(),
                Type::Byte => function_compiler.next_byte_heap_index(),
                Type::Character => function_compiler.next_character_heap_index(),
                Type::Float => function_compiler.next_float_heap_index(),
                Type::Integer => function_compiler.next_integer_heap_index(),
                Type::String => function_compiler.next_string_heap_index(),
                Type::List(_) => function_compiler.next_list_heap_index(),
                Type::Function(_) | Type::FunctionSelf => {
                    function_compiler.next_function_heap_index()
                }
                _ => todo!(),
            };
            let address = Address::heap(local_memory_index);

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

        let function_end = function_compiler.previous_position.1;
        let prototype_index = function_compiler.prototype_index;
        let chunk = function_compiler.finish();
        let memory_index = self.next_function_heap_index();
        let address = Address::heap(memory_index);
        let load_function = Instruction::load(
            Address::heap(memory_index),
            Address::constant(prototype_index),
            OperandType::FUNCTION,
            false,
        );
        let r#type = Type::Function(Box::new(chunk.r#type.clone()));

        if let Some(identifier) = identifier {
            self.declare_local(
                identifier,
                address,
                r#type.clone(),
                false,
                self.current_block_scope,
            );
        }

        self.prototypes.push(Arc::new(chunk));
        self.emit_instruction(load_function, r#type, Span(function_start, function_end));

        Ok(())
    }

    fn parse_call(&mut self) -> Result<(), CompileError> {
        let start = self.previous_position.0;

        self.advance()?;

        let (last_instruction, last_instruction_type, _) =
            self.instructions
                .pop()
                .ok_or_else(|| CompileError::ExpectedExpression {
                    found: self.previous_token.to_owned(),
                    position: self.previous_position,
                })?;
        let function_return_type = match &last_instruction_type {
            Type::Function(function_type) => function_type.return_type.clone(),
            Type::FunctionSelf => self.r#type.return_type.clone(),
            _ => {
                return Err(CompileError::ExpectedFunction {
                    found: self.previous_token.to_owned(),
                    actual_type: last_instruction_type.clone(),
                    position: self.previous_position,
                });
            }
        };
        let type_argument_list = Vec::new();
        let mut value_argument_list = Vec::new();

        while !self.allow(Token::RightParenthesis)? {
            self.parse_expression()?;
            self.allow(Token::Comma)?;

            let (argument_address, r#type) = match self.get_last_operation() {
                Some(Operation::LOAD) => {
                    let (instruction, instruction_type, _) = self.instructions.pop().unwrap();

                    (instruction.b_address(), instruction_type)
                }
                None => {
                    return Err(CompileError::ExpectedExpression {
                        found: self.previous_token.to_owned(),
                        position: self.previous_position,
                    });
                }
                _ => {
                    let (instruction, instruction_type, _) = self.instructions.last().unwrap();

                    (instruction.destination(), instruction_type.clone())
                }
            };

            value_argument_list.push((argument_address, r#type.kind()));
        }

        let argument_list_index = self.arguments.len() as u16;

        self.arguments
            .push((value_argument_list, type_argument_list));

        let end = self.current_position.1;
        let (destination, return_operand_type) = match function_return_type {
            Type::None => (Address::default(), OperandType::NONE),
            Type::Boolean => (
                Address::heap(self.next_boolean_heap_index()),
                OperandType::BOOLEAN,
            ),
            Type::Byte => (
                Address::heap(self.next_byte_heap_index()),
                OperandType::BYTE,
            ),
            Type::Character => (
                Address::heap(self.next_character_heap_index()),
                OperandType::CHARACTER,
            ),
            Type::Float => (
                Address::heap(self.next_float_heap_index()),
                OperandType::FLOAT,
            ),
            Type::Integer => (
                Address::heap(self.next_integer_heap_index()),
                OperandType::INTEGER,
            ),
            Type::String => (
                Address::heap(self.next_string_heap_index()),
                OperandType::STRING,
            ),
            Type::List(_) => (
                Address::heap(self.next_list_heap_index()),
                OperandType::LIST,
            ),
            Type::Function(_) => (
                Address::heap(self.next_function_heap_index()),
                OperandType::FUNCTION,
            ),
            Type::FunctionSelf => (Address::function_self(), OperandType::FUNCTION_SELF),
            _ => todo!(),
        };
        let call = Instruction::call(
            destination,
            last_instruction.b_address(),
            argument_list_index,
            return_operand_type,
        );

        self.emit_instruction(call, function_return_type, Span(start, end));

        Ok(())
    }

    fn parse_call_native(
        &mut self,
        function: NativeFunction,
        start: Span,
    ) -> Result<(), CompileError> {
        let mut type_argument_list = Vec::new();

        if self.allow(Token::Less)? {
            while !self.allow(Token::Greater)? {
                let r#type = self.parse_type()?;

                type_argument_list.push(r#type);

                self.allow(Token::Comma)?;
            }
        }

        self.expect(Token::LeftParenthesis)?;

        let mut value_argument_list = Vec::new();

        while !self.allow(Token::RightParenthesis)? {
            self.parse_expression()?;
            self.allow(Token::Comma)?;

            let (argument_address, r#type) = match self.get_last_operation() {
                Some(Operation::LOAD) => {
                    let (instruction, instruction_type, _) = self.instructions.pop().unwrap();

                    (instruction.b_address(), instruction_type)
                }
                None => {
                    return Err(CompileError::ExpectedExpression {
                        found: self.previous_token.to_owned(),
                        position: self.previous_position,
                    });
                }
                _ => {
                    let (instruction, instruction_type, _) = self.instructions.last().unwrap();

                    (instruction.destination(), instruction_type.clone())
                }
            };

            value_argument_list.push((argument_address, r#type.kind()));
        }

        let argument_list_index = self.arguments.len() as u16;

        self.arguments
            .push((value_argument_list, type_argument_list));

        let end = self.current_position.1;
        let return_type = function.r#type().return_type.clone();
        let destination_index = match return_type {
            Type::None => 0,
            Type::Boolean => self.next_boolean_heap_index(),
            Type::Byte => self.next_byte_heap_index(),
            Type::Character => self.next_character_heap_index(),
            Type::Float => self.next_float_heap_index(),
            Type::Integer => self.next_integer_heap_index(),
            Type::String => self.next_string_heap_index(),
            _ => todo!(),
        };
        let call_native = Instruction::call_native(
            Address::heap(destination_index),
            function,
            argument_list_index,
        );

        self.emit_instruction(call_native, return_type, Span(start.0, end));

        Ok(())
    }

    fn parse_semicolon(&mut self) -> Result<(), CompileError> {
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

            text
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
                module: Module::new(),
                name,
            },
        );

        self.expect(Token::LeftBrace)?;
        self.parse_items()?;
        self.expect(Token::RightBrace)?;

        let end = self.previous_position.1;
        let position = Span(start, end);

        if let CompileMode::Module {
            module: new_module,
            name: new_module_name,
        } = replace(&mut self.mode, old_mode)
        {
            let new_module_path =
                Path::new_borrowed(new_module_name).ok_or(CompileError::InvalidPath {
                    found: new_module_name.to_string(),
                    position: self.previous_position,
                })?;

            self.main_module.items.insert(
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
                    self.parse_function()?;

                    let (_, _, function_position) = self.instructions.pop().unwrap();
                    let prototype = self.prototypes.pop().unwrap();

                    if let CompileMode::Module { module, .. } = &mut self.mode {
                        let function_name = prototype.name.as_ref().unwrap();
                        let path = Path::new_owned(function_name.to_string()).ok_or_else(|| {
                            CompileError::InvalidPath {
                                found: function_name.to_string(),
                                position: self.previous_position,
                            }
                        })?;

                        module
                            .items
                            .insert(path, (Item::Function(prototype), function_position));
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

            Path::new_borrowed(text).ok_or(CompileError::InvalidPath {
                found: text.to_string(),
                position: self.current_position,
            })?
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

        let value = match r#type.kind() {
            TypeKind::Boolean => {
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

                ConcreteValue::Boolean(boolean)
            }
            TypeKind::Byte => {
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

                ConcreteValue::Byte(byte)
            }
            TypeKind::Character => {
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

                ConcreteValue::Character(character)
            }
            TypeKind::Float => {
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

                ConcreteValue::Float(float)
            }
            TypeKind::Integer => {
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

                ConcreteValue::Integer(integer)
            }
            TypeKind::String => {
                let string = if let Token::String(text) = self.current_token {
                    self.advance()?;

                    DustString::from(text)
                } else {
                    return Err(CompileError::ExpectedToken {
                        expected: TokenKind::String,
                        found: self.current_token.to_owned(),
                        position: self.current_position,
                    });
                };

                ConcreteValue::String(string)
            }
            _ => todo!(),
        };
        let end = self.current_position.1;
        let position = Span(start, end);

        if let CompileMode::Module { module, .. } = &mut self.mode {
            module.items.insert(path, (Item::Constant(value), position));
        } else {
            self.main_module
                .items
                .insert(path, (Item::Constant(value), position));
        }

        self.allow(Token::Semicolon)?;

        Ok(())
    }

    fn parse_use(&mut self) -> Result<(), CompileError> {
        self.advance()?;

        let item_path = if let Token::Identifier(text) = self.current_token {
            self.advance()?;

            Path::new_borrowed(text).ok_or(CompileError::InvalidPath {
                found: text.to_string(),
                position: self.previous_position,
            })?
        } else {
            return Err(CompileError::ExpectedToken {
                expected: TokenKind::Identifier,
                found: self.current_token.to_owned(),
                position: self.current_position,
            });
        };

        self.allow(Token::Semicolon)?;

        let mut active_module = if let CompileMode::Module { module, .. } = &self.mode {
            module
        } else {
            &*self.main_module
        };

        let module_names = item_path.module_names();

        for module_name in module_names {
            if let Some((Item::Module(module), _)) = active_module
                .items
                .get(&Path::new_borrowed(module_name).unwrap())
            {
                active_module = module;
            } else {
                return Err(CompileError::UnknownModule {
                    module_name: module_name.to_string(),
                    position: self.previous_position,
                });
            }
        }

        let (item, item_position) = {
            let (item, position) = if let Some(found) = find_item!(item_path, &*self.main_module) {
                found
            } else {
                return Err(CompileError::UnknownItem {
                    item_name: item_path.item_name().to_string(),
                    position: self.previous_position,
                });
            };

            (item, position)
        };

        self.bring_item_into_local_scope(item_path.item_name(), item, item_position);

        Ok(())
    }

    fn bring_item_into_local_scope(
        &mut self,
        item_name: &str,
        item: Item,
        item_position: Span,
    ) -> (Address, Type) {
        match item {
            Item::Constant(value) => {
                let (instruction, destination_address, r#type) = match value {
                    ConcreteValue::Boolean(boolean) => {
                        let memory_index = self.next_boolean_heap_index();
                        let destination = Address::heap(memory_index);
                        let operand = Address::new(boolean as u16, MemoryKind::default());
                        let instruction =
                            Instruction::load(destination, operand, OperandType::BOOLEAN, false);

                        (instruction, destination, Type::Boolean)
                    }
                    ConcreteValue::Byte(byte) => {
                        let memory_index = self.next_byte_heap_index();
                        let destination = Address::heap(memory_index);
                        let operand = Address::new(byte as u16, MemoryKind::default());
                        let instruction =
                            Instruction::load(destination, operand, OperandType::BYTE, false);

                        (instruction, destination, Type::Byte)
                    }
                    ConcreteValue::Character(character) => {
                        let memory_index = self.next_character_heap_index();
                        let destination = Address::heap(memory_index);
                        let constant_index = self.push_or_get_constant_character(character);
                        let operand = Address::constant(constant_index);
                        let instruction =
                            Instruction::load(destination, operand, OperandType::CHARACTER, false);

                        (instruction, destination, Type::Character)
                    }
                    ConcreteValue::Float(float) => {
                        let memory_index = self.next_float_heap_index();
                        let destination = Address::heap(memory_index);
                        let constant_index = self.push_or_get_constant_float(float);
                        let operand = Address::constant(constant_index);
                        let instruction =
                            Instruction::load(destination, operand, OperandType::FLOAT, false);

                        (instruction, destination, Type::Float)
                    }
                    ConcreteValue::Integer(integer) => {
                        let memory_index = self.next_integer_heap_index();
                        let destination = Address::heap(memory_index);
                        let constant_index = self.push_or_get_constant_integer(integer);
                        let operand = Address::constant(constant_index);
                        let instruction =
                            Instruction::load(destination, operand, OperandType::INTEGER, false);

                        (instruction, destination, Type::Integer)
                    }
                    ConcreteValue::String(string) => {
                        let memory_index = self.next_string_heap_index();
                        let destination = Address::heap(memory_index);
                        let constant_index = self.push_or_get_constant_string(string);
                        let operand = Address::constant(constant_index);
                        let instruction =
                            Instruction::load(destination, operand, OperandType::STRING, false);

                        (instruction, destination, Type::String)
                    }
                    _ => todo!("Handle other constant types in use statement"),
                };

                self.emit_instruction(instruction, Type::None, item_position);
                self.declare_local(
                    item_name,
                    destination_address,
                    r#type.clone(),
                    false,
                    self.current_block_scope,
                );

                (destination_address, r#type)
            }
            Item::Function(prototype) => {
                let prototype_index = self.prototypes.len() as u16;

                self.prototypes.push(Arc::clone(&prototype));

                let address = Address::constant(prototype_index);
                let r#type = Type::Function(Box::new(prototype.r#type.clone()));

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
                let aliased_path = Path::new_borrowed("std::convert::int_to_str").unwrap();
                let (item, item_position) = find_item!(aliased_path, &*self.main_module)
                    .ok_or_else(|| CompileError::UnknownItem {
                        item_name: aliased_path.to_string(),
                        position: self.previous_position,
                    })?;
                let (variable_address, item_type) =
                    self.bring_item_into_local_scope(identifier, item, item_position);
                let destination = Address::heap(self.next_function_heap_index());
                let load =
                    Instruction::load(destination, variable_address, OperandType::FUNCTION, false);

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
        if let Some(prefix_parser) = ParseRule::from(&self.current_token).prefix {
            debug!(
                "{} is prefix with precedence {precedence}",
                self.current_token,
            );

            prefix_parser(self)?;
        }

        let mut infix_rule = ParseRule::from(&self.current_token);

        while precedence <= infix_rule.precedence {
            if let Some(infix_parser) = infix_rule.infix {
                debug!(
                    "{} is infix with precedence {precedence}",
                    self.current_token,
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

            infix_rule = ParseRule::from(&self.current_token);
        }

        Ok(())
    }
}
