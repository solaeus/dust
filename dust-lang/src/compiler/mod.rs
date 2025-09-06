mod error;

#[cfg(test)]
mod tests;

pub use error::CompileError;

use std::{cell::RefCell, collections::HashMap, rc::Rc};

use rustc_hash::FxBuildHasher;
use tracing::{Level, debug, info, span};

use crate::{
    Address, Chunk, FunctionType, Instruction, OperandType, Operation, Resolver,
    dust_crate::Program,
    dust_error::DustError,
    parser::{ParseResult, Parser},
    resolver::{DeclarationId, DeclarationKind, ScopeId, TypeId},
    syntax_tree::{SyntaxId, SyntaxKind, SyntaxNode, SyntaxTree},
};

pub fn compile_main(source: &'_ str) -> Result<Chunk, DustError<'_>> {
    let parser = Parser::new();
    let ParseResult {
        syntax_tree,
        resolver,
        errors,
    } = parser.parse_once(source, ScopeId::MAIN);

    if !errors.is_empty() {
        return Err(DustError::parse(errors, source));
    }

    let chunk_compiler = ChunkCompiler::new(&syntax_tree, resolver);

    chunk_compiler
        .compile()
        .map_err(|error| DustError::compile(error, source))
}

pub struct Compiler {
    _allow_native_functions: bool,
}

impl Compiler {
    pub fn new(allow_native_functions: bool) -> Self {
        Self {
            _allow_native_functions: allow_native_functions,
        }
    }

    pub fn compile<'src>(&self, sources: &[(&str, &'src str)]) -> Result<Program, DustError<'src>> {
        let prototypes = Rc::new(RefCell::new(Vec::new()));

        let mut parser = Parser::new();
        let ParseResult {
            syntax_tree,
            resolver,
            errors,
        } = parser.parse(sources[0].1, ScopeId::MAIN);

        if !errors.is_empty() {
            return Err(DustError::parse(errors, sources[0].1));
        }

        prototypes.borrow_mut().push(Chunk::default());

        let chunk_compiler = ChunkCompiler::new(&syntax_tree, resolver);
        let main_chunk = chunk_compiler
            .compile()
            .map_err(|error| DustError::compile(error, sources[0].1))?;

        prototypes.borrow_mut()[0] = main_chunk;

        let prototypes = Rc::into_inner(prototypes)
            .expect("Unneccessary borrow of 'prototypes'")
            .into_inner();

        Ok(Program {
            prototypes,
            cell_count: 0,
        })
    }
}

#[derive(Debug)]
pub struct ChunkCompiler<'a> {
    /// Target syntax tree for compilation.
    syntax_tree: &'a SyntaxTree,

    /// Context for modules, types and declarations provided by the parser.
    resolver: Resolver,

    /// Bytecode instruction collection that is filled during compilation.
    instructions: Vec<Instruction>,

    /// Concatenated list of arguments referenced by CALL instructions.
    call_arguments: Vec<(Address, OperandType)>,

    /// Concatenated list of register indexes that are referenced by DROP instructions.
    drop_lists: Vec<u16>,

    /// Apparent return type of the function being compiled. This field is modified during compilation
    /// to reflect the actual return type of the function
    return_type: TypeId,

    /// Index of the the chunk being compiled in the program's prototype list. For the main function,
    /// this is 0 as a default but the main chunk is actually the last one in the list.
    prototype_index: u16,

    /// Local variables registers and a boolean indicating if they are mutable.
    locals: HashMap<DeclarationId, Local, FxBuildHasher>,

    minimum_register: u16,
}

impl<'a> ChunkCompiler<'a> {
    pub fn new(syntax_tree: &'a SyntaxTree, resolver: Resolver) -> Self {
        Self {
            syntax_tree,
            resolver,
            instructions: Vec::new(),
            call_arguments: Vec::new(),
            drop_lists: Vec::new(),
            return_type: TypeId::NONE,
            prototype_index: 0,
            locals: HashMap::default(),
            minimum_register: 0,
        }
    }

    pub fn compile(mut self) -> Result<Chunk, CompileError> {
        let span = span!(Level::INFO, "Compiling");
        let _enter = span.enter();

        self.compile_item(SyntaxId(0))?;

        let return_type =
            self.resolver
                .resolve_type(self.return_type)
                .ok_or(CompileError::MissingType {
                    type_id: self.return_type,
                })?;
        let register_count = self.get_next_register();

        self.resolver.constants.trim_string_pool();

        Ok(Chunk {
            name: Some("main".to_string()),
            r#type: FunctionType::new([], [], return_type),
            instructions: self.instructions,
            constants: self.resolver.constants,
            call_arguments: self.call_arguments,
            drop_lists: self.drop_lists,
            register_count,
            prototype_index: self.prototype_index,
        })
    }

    fn get_next_register(&mut self) -> u16 {
        self.instructions
            .iter()
            .fold(self.minimum_register, |acc, instruction| {
                if instruction.yields_value() {
                    acc.max(instruction.destination().index + 1)
                } else {
                    acc
                }
            })
    }

    fn get_constant_address(&mut self, constant: Constant) -> Address {
        let index = match constant {
            Constant::Boolean(boolean) => return Address::encoded(boolean as u16),
            Constant::Byte(byte) => return Address::encoded(byte as u16),
            Constant::Character(character) => self.resolver.constants.add_character(character),
            Constant::Float(float) => self.resolver.constants.add_float(float),
            Constant::Integer(integer) => self.resolver.constants.add_integer(integer),
            Constant::String {
                pool_start,
                pool_end,
            } => self
                .resolver
                .constants
                .add_pooled_string(pool_start, pool_end),
        };

        Address::constant(index)
    }

    fn combine_constants(
        &mut self,
        left: Constant,
        right: Constant,
        operation: SyntaxKind,
    ) -> Result<Constant, CompileError> {
        debug!(
            "Combining constants: {:?} {:?} {:?}",
            left, right, operation
        );

        let combined = match (left, right) {
            (Constant::Boolean(left), Constant::Boolean(right)) => match operation {
                SyntaxKind::AndExpression => Constant::Boolean(left && right),
                SyntaxKind::OrExpression => Constant::Boolean(left || right),
                SyntaxKind::GreaterThanExpression => Constant::Boolean(left && right),
                SyntaxKind::GreaterThanOrEqualExpression => Constant::Boolean(left >= right),
                SyntaxKind::LessThanExpression => Constant::Boolean(left || right),
                SyntaxKind::LessThanOrEqualExpression => Constant::Boolean(left <= right),
                SyntaxKind::EqualExpression => Constant::Boolean(left == right),
                SyntaxKind::NotEqualExpression => Constant::Boolean(left != right),
                _ => todo!(),
            },
            (Constant::Byte(left), Constant::Byte(right)) => match operation {
                SyntaxKind::AdditionExpression => Constant::Byte(left.saturating_add(right)),
                SyntaxKind::SubtractionExpression => Constant::Byte(left.saturating_sub(right)),
                SyntaxKind::MultiplicationExpression => Constant::Byte(left.saturating_mul(right)),
                SyntaxKind::DivisionExpression => Constant::Byte(left.saturating_div(right)),
                SyntaxKind::ModuloExpression => Constant::Byte(left % right),
                SyntaxKind::GreaterThanExpression => Constant::Boolean(left > right),
                SyntaxKind::GreaterThanOrEqualExpression => Constant::Boolean(left >= right),
                SyntaxKind::LessThanExpression => Constant::Boolean(left < right),
                SyntaxKind::LessThanOrEqualExpression => Constant::Boolean(left <= right),
                SyntaxKind::EqualExpression => Constant::Boolean(left == right),
                SyntaxKind::NotEqualExpression => Constant::Boolean(left != right),
                _ => todo!(),
            },
            (Constant::Float(left), Constant::Float(right)) => match operation {
                SyntaxKind::AdditionExpression => Constant::Float(left + right),
                SyntaxKind::SubtractionExpression => Constant::Float(left - right),
                SyntaxKind::MultiplicationExpression => Constant::Float(left * right),
                SyntaxKind::DivisionExpression => Constant::Float(left / right),
                SyntaxKind::ModuloExpression => Constant::Float(left % right),
                SyntaxKind::GreaterThanExpression => Constant::Boolean(left > right),
                SyntaxKind::GreaterThanOrEqualExpression => Constant::Boolean(left >= right),
                SyntaxKind::LessThanExpression => Constant::Boolean(left < right),
                SyntaxKind::LessThanOrEqualExpression => Constant::Boolean(left <= right),
                SyntaxKind::EqualExpression => Constant::Boolean(left == right),
                SyntaxKind::NotEqualExpression => Constant::Boolean(left != right),
                _ => todo!(),
            },
            (Constant::Integer(left), Constant::Integer(right)) => match operation {
                SyntaxKind::AdditionExpression => Constant::Integer(left.saturating_add(right)),
                SyntaxKind::SubtractionExpression => Constant::Integer(left.saturating_sub(right)),
                SyntaxKind::MultiplicationExpression => {
                    Constant::Integer(left.saturating_mul(right))
                }
                SyntaxKind::DivisionExpression => Constant::Integer(left.saturating_div(right)),
                SyntaxKind::ModuloExpression => Constant::Integer(left % right),
                SyntaxKind::GreaterThanExpression => Constant::Boolean(left > right),
                SyntaxKind::GreaterThanOrEqualExpression => Constant::Boolean(left >= right),
                SyntaxKind::LessThanExpression => Constant::Boolean(left < right),
                SyntaxKind::LessThanOrEqualExpression => Constant::Boolean(left <= right),
                SyntaxKind::EqualExpression => Constant::Boolean(left == right),
                SyntaxKind::NotEqualExpression => Constant::Boolean(left != right),
                _ => todo!(),
            },
            (Constant::Character(left), Constant::Character(right)) => match operation {
                SyntaxKind::AdditionExpression => {
                    let mut string = String::with_capacity(2);

                    string.push(left);
                    string.push(right);

                    let combined = self.resolver.constants.push_str_to_string_pool(&string);

                    Constant::String {
                        pool_start: combined.0,
                        pool_end: combined.1,
                    }
                }
                SyntaxKind::GreaterThanExpression => Constant::Boolean(left > right),
                SyntaxKind::GreaterThanOrEqualExpression => Constant::Boolean(left >= right),
                SyntaxKind::LessThanExpression => Constant::Boolean(left < right),
                SyntaxKind::LessThanOrEqualExpression => Constant::Boolean(left <= right),
                SyntaxKind::EqualExpression => Constant::Boolean(left == right),
                SyntaxKind::NotEqualExpression => Constant::Boolean(left != right),
                _ => todo!("Error"),
            },
            (
                Constant::String {
                    pool_start: left_pool_start,
                    pool_end: left_pool_end,
                },
                Constant::String {
                    pool_start: right_pool_start,
                    pool_end: right_pool_end,
                },
            ) => {
                let left = self
                    .resolver
                    .constants
                    .get_string_pool(left_pool_start as usize..left_pool_end as usize);
                let right = self
                    .resolver
                    .constants
                    .get_string_pool(right_pool_start as usize..right_pool_end as usize);

                match operation {
                    SyntaxKind::AdditionExpression => {
                        if left_pool_end == right_pool_start {
                            return Ok(Constant::String {
                                pool_start: left_pool_start,
                                pool_end: right_pool_end,
                            });
                        }

                        let mut string = String::with_capacity(left.len() + right.len());

                        string.push_str(left);
                        string.push_str(right);

                        let combined = self.resolver.constants.push_str_to_string_pool(&string);

                        Constant::String {
                            pool_start: combined.0,
                            pool_end: combined.1,
                        }
                    }
                    SyntaxKind::GreaterThanExpression => Constant::Boolean(left > right),
                    SyntaxKind::GreaterThanOrEqualExpression => Constant::Boolean(left >= right),
                    SyntaxKind::LessThanExpression => Constant::Boolean(left < right),
                    SyntaxKind::LessThanOrEqualExpression => Constant::Boolean(left <= right),
                    SyntaxKind::EqualExpression => Constant::Boolean(left == right),
                    SyntaxKind::NotEqualExpression => Constant::Boolean(left != right),
                    _ => todo!("Error"),
                }
            }
            (
                Constant::Character(left),
                Constant::String {
                    pool_start,
                    pool_end,
                },
            ) => {
                let right = self
                    .resolver
                    .constants
                    .get_string_pool(pool_start as usize..pool_end as usize);
                let mut string = String::with_capacity(1 + right.len());

                string.push(left);
                string.push_str(right);

                let combined = match operation {
                    SyntaxKind::AdditionExpression => {
                        self.resolver.constants.push_str_to_string_pool(&string)
                    }
                    _ => todo!("Error"),
                };

                Constant::String {
                    pool_start: combined.0,
                    pool_end: combined.1,
                }
            }
            (
                Constant::String {
                    pool_start,
                    pool_end,
                },
                Constant::Character(right),
            ) => {
                let left = self
                    .resolver
                    .constants
                    .get_string_pool(pool_start as usize..pool_end as usize);
                let mut string = String::with_capacity(left.len() + 1);

                string.push_str(left);
                string.push(right);

                let combined = match operation {
                    SyntaxKind::AdditionExpression => {
                        self.resolver.constants.push_str_to_string_pool(&string)
                    }
                    _ => todo!("Error"),
                };

                Constant::String {
                    pool_start: combined.0,
                    pool_end: combined.1,
                }
            }
            _ => todo!(),
        };

        Ok(combined)
    }

    fn handle_operand(&mut self, instruction: Instruction) -> Address {
        match instruction.operation() {
            Operation::LOAD => instruction.b_address(),
            _ => {
                self.instructions.push(instruction);

                instruction.destination()
            }
        }
    }

    fn compile_item(&mut self, node_id: SyntaxId) -> Result<(), CompileError> {
        let node = *self
            .syntax_tree
            .get_node(node_id)
            .ok_or(CompileError::MissingSyntaxNode { id: node_id })?;

        match node.kind {
            SyntaxKind::MainFunctionItem => self.compile_main_function_statement(&node),
            _ => Err(CompileError::ExpectedItem {
                node_kind: node.kind,
                position: node.position,
            }),
        }
    }

    fn compile_statement(&mut self, node_id: SyntaxId) -> Result<(), CompileError> {
        let node = *self
            .syntax_tree
            .get_node(node_id)
            .ok_or(CompileError::MissingSyntaxNode { id: node_id })?;

        match node.kind {
            SyntaxKind::LetStatement | SyntaxKind::LetMutStatement => {
                self.compile_let_statement(&node)
            }
            SyntaxKind::FunctionStatement => todo!("Compile function statement"),
            SyntaxKind::ExpressionStatement => self.compile_expression_statement(&node),
            SyntaxKind::SemicolonStatement => todo!("Compile semicolon statement"),
            _ => Err(CompileError::ExpectedStatement {
                node_kind: node.kind,
                position: node.position,
            }),
        }
    }

    fn compile_main_function_statement(&mut self, node: &SyntaxNode) -> Result<(), CompileError> {
        info!("Compiling main function");

        let (start_children, child_count) = (node.children.0 as usize, node.children.1 as usize);
        let end_children = start_children + child_count;

        let mut current_child_index = start_children;

        while current_child_index < end_children {
            let child_id = *self.syntax_tree.children.get(current_child_index).ok_or(
                CompileError::MissingChild {
                    parent_kind: node.kind,
                    child_index: current_child_index as u32,
                },
            )?;
            current_child_index += 1;

            if current_child_index == end_children {
                let child_node = self
                    .syntax_tree
                    .get_node(child_id)
                    .ok_or(CompileError::MissingSyntaxNode { id: child_id })?;

                if child_node.kind.is_item() {
                    self.compile_item(child_id)?;

                    let return_instruction =
                        Instruction::r#return(false, Address::default(), OperandType::NONE);

                    self.instructions.push(return_instruction);
                } else if child_node.kind.is_statement() {
                    self.compile_statement(child_id)?;

                    let return_instruction =
                        Instruction::r#return(false, Address::default(), OperandType::NONE);

                    self.instructions.push(return_instruction);
                } else {
                    let return_emission = self.compile_expression(child_id, child_node)?;
                    let (return_operand, return_type, return_operand_type) = match return_emission {
                        Emission::Instruction(instruction, r#type) => {
                            let operand_type = self
                                .resolver
                                .resolve_type(r#type)
                                .ok_or(CompileError::MissingType { type_id: r#type })?
                                .as_operand_type();
                            let mut return_operand = self.handle_operand(instruction);

                            if let Some(last_instruction) = self.instructions.last()
                                && last_instruction.operation() == Operation::LOAD
                                && last_instruction.destination() == return_operand
                            {
                                return_operand = last_instruction.b_address();

                                self.instructions.pop();
                            }

                            (return_operand, r#type, operand_type)
                        }
                        Emission::Instructions(instructions, r#type) => {
                            let last_instruction = instructions.last().unwrap();
                            let operand_type = self
                                .resolver
                                .resolve_type(r#type)
                                .ok_or(CompileError::MissingType { type_id: r#type })?
                                .as_operand_type();

                            self.instructions.extend(instructions.iter());

                            (last_instruction.destination(), r#type, operand_type)
                        }
                        Emission::Constant(constant) => {
                            let (r#type, operand_type) = match constant {
                                Constant::Boolean(_) => (TypeId::BOOLEAN, OperandType::BOOLEAN),
                                Constant::Byte(_) => (TypeId::BYTE, OperandType::BYTE),
                                Constant::Character(_) => {
                                    (TypeId::CHARACTER, OperandType::CHARACTER)
                                }
                                Constant::Float(_) => (TypeId::FLOAT, OperandType::FLOAT),
                                Constant::Integer(_) => (TypeId::INTEGER, OperandType::INTEGER),
                                Constant::String { .. } => (TypeId::STRING, OperandType::STRING),
                            };
                            let address = self.get_constant_address(constant);
                            let destination = Address::register(self.get_next_register());
                            let instruction =
                                Instruction::load(destination, address, operand_type, false);
                            let return_operand = self.handle_operand(instruction);

                            (return_operand, r#type, operand_type)
                        }
                    };

                    self.return_type = return_type;
                    let return_instruction = Instruction::r#return(
                        return_operand_type != OperandType::NONE,
                        return_operand,
                        return_operand_type,
                    );

                    self.instructions.push(return_instruction);
                }
            } else {
                self.compile_statement(child_id)?;
            }
        }

        Ok(())
    }

    fn compile_expression_statement(&mut self, node: &SyntaxNode) -> Result<(), CompileError> {
        info!("Compiling expression statement");

        let expression_id = SyntaxId(node.children.0);
        let expression_node = self
            .syntax_tree
            .get_node(expression_id)
            .ok_or(CompileError::MissingSyntaxNode { id: expression_id })?;
        let expression_emission = self.compile_expression(expression_id, expression_node)?;

        match expression_emission {
            Emission::Instruction(instruction, _) => {
                self.instructions.push(instruction);
            }
            Emission::Instructions(instructions, _) => {
                self.instructions.extend(instructions.iter());
            }
            Emission::Constant(constant) => {
                let r#type = constant.operand_type();
                let address = self.get_constant_address(constant);
                let destination = Address::register(self.get_next_register());
                let load_instruction = Instruction::load(destination, address, r#type, false);

                self.instructions.push(load_instruction);
            }
        }

        Ok(())
    }

    fn compile_let_statement(&mut self, node: &SyntaxNode) -> Result<(), CompileError> {
        info!("Compiling let statement");

        let declaration_id = DeclarationId(node.payload);
        let expression_statement_id = SyntaxId(node.children.1);

        let declaration = self
            .resolver
            .get_declaration_from_id(declaration_id)
            .ok_or(CompileError::MissingDeclaration { id: declaration_id })?;
        let is_mutable = declaration.kind == DeclarationKind::LocalMutable;
        let expression_statement_node = self.syntax_tree.get_node(expression_statement_id).ok_or(
            CompileError::MissingSyntaxNode {
                id: expression_statement_id,
            },
        )?;
        let expression_id = SyntaxId(expression_statement_node.children.0);
        let expression_node = self
            .syntax_tree
            .get_node(expression_id)
            .ok_or(CompileError::MissingSyntaxNode { id: expression_id })?;
        let expression_emission = self.compile_expression(expression_id, expression_node)?;
        let destination_register = match expression_emission {
            Emission::Instruction(instruction, _) => {
                let destination_register = instruction.destination().index;

                self.instructions.push(instruction);

                destination_register
            }
            Emission::Instructions(instructions, _) => {
                let first_instruction = instructions[0];
                let destination_register = first_instruction.destination().index;

                self.instructions.extend(instructions.iter());

                destination_register
            }
            Emission::Constant(constant) => {
                let r#type = constant.operand_type();
                let address = self.get_constant_address(constant);
                let destination = Address::register(self.get_next_register());
                let instruction = Instruction::load(destination, address, r#type, false);

                self.instructions.push(instruction);

                destination.index
            }
        };

        self.locals.insert(
            declaration_id,
            Local {
                register: destination_register,
                is_mutable,
            },
        );

        Ok(())
    }

    fn compile_expression(
        &mut self,
        node_id: SyntaxId,
        node: &SyntaxNode,
    ) -> Result<Emission, CompileError> {
        match node.kind {
            SyntaxKind::BooleanExpression => self.compile_boolean_expression(node),
            SyntaxKind::ByteExpression => self.compile_byte_expression(node),
            SyntaxKind::CharacterExpression => self.compile_character_expression(node_id, node),
            SyntaxKind::FloatExpression => self.compile_float_expression(node),
            SyntaxKind::IntegerExpression => self.compile_integer_expression(node),
            SyntaxKind::StringExpression => self.compile_string_expression(node),
            SyntaxKind::AdditionExpression
            | SyntaxKind::SubtractionExpression
            | SyntaxKind::MultiplicationExpression
            | SyntaxKind::DivisionExpression
            | SyntaxKind::ModuloExpression => self.compile_math_expression(node_id, node),
            SyntaxKind::GreaterThanExpression
            | SyntaxKind::GreaterThanOrEqualExpression
            | SyntaxKind::LessThanExpression
            | SyntaxKind::LessThanOrEqualExpression
            | SyntaxKind::EqualExpression
            | SyntaxKind::NotEqualExpression => self.compile_comparison_expression(node_id, node),
            SyntaxKind::AndExpression | SyntaxKind::OrExpression => {
                self.compile_logical_expression(node_id, node)
            }
            SyntaxKind::NotExpression | SyntaxKind::NegationExpression => {
                self.compile_unary_expression(node_id, node)
            }
            SyntaxKind::GroupedExpression => self.compile_grouped_expression(node_id, node),
            SyntaxKind::PathExpression => self.compile_path_expression(node_id, node),
            _ => Err(CompileError::ExpectedExpression {
                node_kind: node.kind,
                position: node.position,
            }),
        }
    }

    fn compile_boolean_expression(&mut self, node: &SyntaxNode) -> Result<Emission, CompileError> {
        info!("Compiling boolean expression");

        Ok(Emission::Constant(Constant::Boolean(node.children.0 != 0)))
    }

    fn compile_byte_expression(&mut self, node: &SyntaxNode) -> Result<Emission, CompileError> {
        info!("Compiling byte expression");

        Ok(Emission::Constant(Constant::Byte(node.children.0 as u8)))
    }

    fn compile_character_expression(
        &mut self,
        _node_id: SyntaxId,
        node: &SyntaxNode,
    ) -> Result<Emission, CompileError> {
        info!("Compiling character expression");

        let character = char::from_u32(node.children.0).unwrap_or_default();

        Ok(Emission::Constant(Constant::Character(character)))
    }

    fn compile_float_expression(&mut self, node: &SyntaxNode) -> Result<Emission, CompileError> {
        info!("Compiling float expression");

        let float = SyntaxNode::decode_float(node.children);

        Ok(Emission::Constant(Constant::Float(float)))
    }

    fn compile_integer_expression(&mut self, node: &SyntaxNode) -> Result<Emission, CompileError> {
        info!("Compiling integer expression");

        let integer = SyntaxNode::decode_integer(node.children);

        Ok(Emission::Constant(Constant::Integer(integer)))
    }

    fn compile_string_expression(&mut self, node: &SyntaxNode) -> Result<Emission, CompileError> {
        info!("Compiling string expression");

        Ok(Emission::Constant(Constant::String {
            pool_start: node.children.0,
            pool_end: node.children.1,
        }))
    }

    fn compile_math_expression(
        &mut self,
        _node_id: SyntaxId,
        node: &SyntaxNode,
    ) -> Result<Emission, CompileError> {
        info!("Compiling math expression");

        let left_index = SyntaxId(node.children.0);
        let left = self.syntax_tree.nodes.get(left_index.0 as usize).ok_or(
            CompileError::MissingChild {
                parent_kind: node.kind,
                child_index: node.children.0,
            },
        )?;
        let right_index = SyntaxId(node.children.1);
        let right = self.syntax_tree.nodes.get(right_index.0 as usize).ok_or(
            CompileError::MissingChild {
                parent_kind: node.kind,
                child_index: node.children.1,
            },
        )?;

        let left_emission = self.compile_expression(left_index, left)?;
        let right_emission = self.compile_expression(right_index, right)?;

        if let (Emission::Constant(left_value), Emission::Constant(right_value)) =
            (&left_emission, &right_emission)
        {
            let combined = self.combine_constants(*left_value, *right_value, node.kind)?;

            return Ok(Emission::Constant(combined));
        }

        let left_address = left_emission.handle_as_operand(self);
        let right_address = right_emission.handle_as_operand(self);

        let operand_type = self
            .resolver
            .resolve_type(TypeId(node.payload))
            .ok_or(CompileError::MissingType {
                type_id: TypeId(node.payload),
            })?
            .as_operand_type();
        let destination = Address::register(self.get_next_register());
        let instruction = match node.kind {
            SyntaxKind::AdditionExpression => {
                Instruction::add(destination, left_address, right_address, operand_type)
            }
            SyntaxKind::SubtractionExpression => {
                Instruction::subtract(destination, left_address, right_address, operand_type)
            }
            SyntaxKind::MultiplicationExpression => {
                Instruction::multiply(destination, left_address, right_address, operand_type)
            }
            SyntaxKind::DivisionExpression => {
                Instruction::divide(destination, left_address, right_address, operand_type)
            }
            SyntaxKind::ModuloExpression => {
                Instruction::modulo(destination, left_address, right_address, operand_type)
            }
            _ => unreachable!("Expected binary expression, found {}", node.kind),
        };

        Ok(Emission::Instruction(instruction, TypeId(node.payload)))
    }

    fn compile_comparison_expression(
        &mut self,
        _node_id: SyntaxId,
        node: &SyntaxNode,
    ) -> Result<Emission, CompileError> {
        let left_index = SyntaxId(node.children.0);
        let left = self.syntax_tree.nodes.get(left_index.0 as usize).ok_or(
            CompileError::MissingChild {
                parent_kind: node.kind,
                child_index: node.children.0,
            },
        )?;
        let right_index = SyntaxId(node.children.1);
        let right = self.syntax_tree.nodes.get(right_index.0 as usize).ok_or(
            CompileError::MissingChild {
                parent_kind: node.kind,
                child_index: node.children.1,
            },
        )?;

        let left_emission = self.compile_expression(left_index, left)?;
        let right_emission = self.compile_expression(right_index, right)?;

        if let (Emission::Constant(left_value), Emission::Constant(right_value)) =
            (&left_emission, &right_emission)
        {
            let combined = self.combine_constants(*left_value, *right_value, node.kind)?;

            return Ok(Emission::Constant(combined));
        }

        let destination = Address::register(self.get_next_register());
        let (left_address, left_type) = match &left_emission {
            Emission::Instruction(instruction, _) => (
                self.handle_operand(*instruction),
                instruction.operand_type(),
            ),
            Emission::Instructions(instructions, _) => {
                self.instructions.extend(instructions);

                (
                    instructions[0].destination(),
                    instructions[0].operand_type(),
                )
            }
            Emission::Constant(constant) => {
                let r#type = constant.operand_type();
                let address = self.get_constant_address(*constant);
                let load_instruction = Instruction::load(destination, address, r#type, false);

                (self.handle_operand(load_instruction), r#type)
            }
        };
        let (right_address, right_type) = match right_emission {
            Emission::Instruction(instruction, _) => {
                (self.handle_operand(instruction), instruction.operand_type())
            }
            Emission::Instructions(mut instructions, _) => {
                let destination = instructions[0].destination();
                let operand_type = instructions[0].operand_type();

                self.instructions.append(&mut instructions);

                (destination, operand_type)
            }
            Emission::Constant(constant) => {
                let r#type = constant.operand_type();
                let address = self.get_constant_address(constant);
                let load_instruction = Instruction::load(destination, address, r#type, false);

                (self.handle_operand(load_instruction), r#type)
            }
        };

        if left_type != right_type {
            todo!("Error");
        }

        let destination = Address::register(self.get_next_register());
        let comparison_instruction = match node.kind {
            SyntaxKind::GreaterThanExpression => {
                Instruction::less_equal(false, left_address, right_address, left_type)
            }
            SyntaxKind::GreaterThanOrEqualExpression => {
                Instruction::less(false, left_address, right_address, left_type)
            }
            SyntaxKind::LessThanExpression => {
                Instruction::less(true, left_address, right_address, left_type)
            }
            SyntaxKind::LessThanOrEqualExpression => {
                Instruction::less_equal(true, left_address, right_address, left_type)
            }
            SyntaxKind::EqualExpression => {
                Instruction::equal(true, left_address, right_address, left_type)
            }
            SyntaxKind::NotEqualExpression => {
                Instruction::equal(false, left_address, right_address, left_type)
            }
            _ => unreachable!("Expected comparison expression, found {}", node.kind),
        };
        let jump_instruction = Instruction::jump(1, true);
        let load_true_instruction = Instruction::load(
            destination,
            Address::encoded(true as u16),
            OperandType::BOOLEAN,
            true,
        );
        let load_false_instruction = Instruction::load(
            destination,
            Address::encoded(false as u16),
            OperandType::BOOLEAN,
            false,
        );

        Ok(Emission::Instructions(
            vec![
                comparison_instruction,
                jump_instruction,
                load_true_instruction,
                load_false_instruction,
            ],
            TypeId::BOOLEAN,
        ))
    }

    fn compile_logical_expression(
        &mut self,
        _node_id: SyntaxId,
        node: &SyntaxNode,
    ) -> Result<Emission, CompileError> {
        info!("Compiling logical expression");

        let left_index = SyntaxId(node.children.0);
        let left = self.syntax_tree.nodes.get(left_index.0 as usize).ok_or(
            CompileError::MissingChild {
                parent_kind: node.kind,
                child_index: node.children.0,
            },
        )?;
        let right_index = SyntaxId(node.children.1);
        let right = self.syntax_tree.nodes.get(right_index.0 as usize).ok_or(
            CompileError::MissingChild {
                parent_kind: node.kind,
                child_index: node.children.1,
            },
        )?;

        let left_emission = self.compile_expression(left_index, left)?;
        let right_emission = self.compile_expression(right_index, right)?;

        if let (Emission::Constant(left_value), Emission::Constant(right_value)) =
            (&left_emission, &right_emission)
        {
            let combined = self.combine_constants(*left_value, *right_value, node.kind)?;

            return Ok(Emission::Constant(combined));
        }

        let left_address = match &left_emission {
            Emission::Instruction(instruction, _) => {
                self.instructions.push(*instruction);

                instruction.destination()
            }
            Emission::Instructions(instructions, _) => {
                self.instructions.extend(instructions.iter());

                instructions.last().unwrap().destination()
            }
            Emission::Constant(constant) => {
                let r#type = constant.operand_type();
                let address = self.get_constant_address(*constant);
                let destination = Address::register(self.get_next_register());
                let load_instruction = Instruction::load(destination, address, r#type, false);

                self.handle_operand(load_instruction)
            }
        };

        let comparator = match node.kind {
            SyntaxKind::AndExpression => true,
            SyntaxKind::OrExpression => false,
            _ => unreachable!("Expected logical expression, found {}", node.kind),
        };
        let test_instruction = Instruction::test(left_address, comparator);
        let jump_instruction = Instruction::jump(1, true);

        self.instructions.push(test_instruction);
        self.instructions.push(jump_instruction);

        let emission = match right_emission {
            Emission::Instruction(instruction, _) => {
                let mut instruction = instruction;

                instruction.set_destination(left_address);

                Emission::Instruction(instruction, TypeId(right.payload))
            }
            Emission::Instructions(instructions, r#type) => {
                Emission::Instructions(instructions, r#type)
            }
            Emission::Constant(constant) => Emission::Constant(constant),
        };

        Ok(emission)
    }

    fn compile_unary_expression(
        &mut self,
        _node_id: SyntaxId,
        node: &SyntaxNode,
    ) -> Result<Emission, CompileError> {
        info!("Compiling unary expression");

        let child_id = SyntaxId(node.children.0);
        let child_node = self
            .syntax_tree
            .get_node(child_id)
            .ok_or(CompileError::MissingChild {
                parent_kind: node.kind,
                child_index: node.children.0,
            })?;
        let child_emission = self.compile_expression(child_id, child_node)?;

        if let Emission::Constant(child_value) = &child_emission {
            let evaluated = match (node.kind, child_value) {
                (SyntaxKind::NotExpression, Constant::Boolean(value)) => Constant::Boolean(!value),
                (SyntaxKind::NegationExpression, Constant::Integer(value)) => {
                    Constant::Integer(-value)
                }
                (SyntaxKind::NegationExpression, Constant::Float(value)) => Constant::Float(-value),
                _ => todo!("Error"),
            };

            return Ok(Emission::Constant(evaluated));
        }

        let child_address = child_emission.handle_as_operand(self);
        let operand_type = self
            .resolver
            .resolve_type(TypeId(node.payload))
            .ok_or(CompileError::MissingType {
                type_id: TypeId(node.payload),
            })?
            .as_operand_type();
        let destination = Address::register(self.get_next_register());
        let negate_instruction = Instruction::negate(destination, child_address, operand_type);

        Ok(Emission::Instruction(
            negate_instruction,
            TypeId(node.payload),
        ))
    }

    fn compile_grouped_expression(
        &mut self,
        node_id: SyntaxId,
        node: &SyntaxNode,
    ) -> Result<Emission, CompileError> {
        info!("Compiling grouped expression");

        let child_id = SyntaxId(node.children.0);
        let child_node = self
            .syntax_tree
            .get_node(child_id)
            .ok_or(CompileError::MissingChild {
                parent_kind: node.kind,
                child_index: node.children.0,
            })?;

        self.compile_expression(node_id, child_node)
    }

    fn compile_path_expression(
        &mut self,
        _node_id: SyntaxId,
        node: &SyntaxNode,
    ) -> Result<Emission, CompileError> {
        info!("Compiling path expression");

        let declaration_id = DeclarationId(node.children.0);
        let destination_register = self
            .locals
            .get(&declaration_id)
            .ok_or(CompileError::MissingLocalRegister { declaration_id })?
            .register;
        let r#type = self
            .resolver
            .resolve_type(TypeId(node.payload))
            .ok_or(CompileError::MissingDeclaration { id: declaration_id })?
            .as_operand_type();
        let load_instruction = Instruction::load(
            Address::register(self.get_next_register()),
            Address::register(destination_register),
            r#type,
            false,
        );

        Ok(Emission::Instruction(
            load_instruction,
            TypeId(node.payload),
        ))
    }

    fn _compile_implicit_return(&mut self, _expression: SyntaxId) -> Result<(), CompileError> {
        // let definition = self.module_resolver.get_definition();
        // let expression_type = self.syntax_tree.resolve_type(expression);
        // let expression_node =
        //     self.syntax_tree
        //         .get_node(expression)
        //         .copied()
        //         .ok_or(CompileError::MissingChild {
        //             parent_kind: SyntaxKind::MainFunctionItem,
        //             child_index: expression,
        //         })?;
        // let expression_instruction = self.compile_expression(expression_node)?;
        // let (returns_value, value_address) = if expression_type == Type::None {
        //     self.instructions.push(expression_instruction);

        //     (false, Address::default())
        // } else {
        //     let address = self.handle_operand(expression_instruction);

        //     (true, address)
        // };
        // let r#return = Instruction::r#return(
        //     returns_value,
        //     value_address,
        //     expression_type.as_operand_type(),
        // );

        // self.instructions.push(r#return);

        Ok(())
    }
}

#[derive(Clone, Debug)]
pub enum Emission {
    Instruction(Instruction, TypeId),
    Instructions(Vec<Instruction>, TypeId),
    Constant(Constant),
}

impl Emission {
    fn handle_as_operand(self, compiler: &mut ChunkCompiler) -> Address {
        match self {
            Emission::Instruction(instruction, _) => compiler.handle_operand(instruction),
            Emission::Instructions(instructions, _) => {
                let first_instruction = instructions[0];
                let destination = first_instruction.destination();

                compiler.instructions.extend(instructions.iter());

                destination
            }
            Emission::Constant(constant) => {
                let r#type = constant.operand_type();
                let address = compiler.get_constant_address(constant);
                let destination = Address::register(compiler.get_next_register());
                let load_instruction = Instruction::load(destination, address, r#type, false);

                compiler.instructions.push(load_instruction);

                compiler.handle_operand(load_instruction)
            }
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum Constant {
    Boolean(bool),
    Byte(u8),
    Character(char),
    Float(f64),
    Integer(i64),
    String { pool_start: u32, pool_end: u32 },
}

impl Constant {
    fn operand_type(&self) -> OperandType {
        match self {
            Constant::Boolean(_) => OperandType::BOOLEAN,
            Constant::Byte(_) => OperandType::BYTE,
            Constant::Character(_) => OperandType::CHARACTER,
            Constant::Float(_) => OperandType::FLOAT,
            Constant::Integer(_) => OperandType::INTEGER,
            Constant::String { .. } => OperandType::STRING,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, PartialOrd, Ord)]
struct Local {
    register: u16,
    is_mutable: bool,
}
