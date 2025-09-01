mod error;
mod local;
// mod fold_constants;

#[cfg(test)]
mod tests;

pub use error::CompileError;
use local::Local;

use tracing::{Level, span};

use crate::{
    Address, Chunk, FunctionType, Instruction, OperandType, Operation, Resolver,
    dust_error::DustError,
    parser::{ParseResult, Parser},
    resolver::{DeclarationId, ScopeId, TypeId},
    syntax_tree::{SyntaxId, SyntaxKind, SyntaxNode, SyntaxTree},
};

pub fn compile(source: &'_ str) -> Result<Chunk, DustError<'_>> {
    let parser = Parser::new();
    let ParseResult {
        syntax_tree,
        resolver,
        errors,
    } = parser.parse_once(source, ScopeId::MAIN);

    if !errors.is_empty() {
        return Err(DustError::parse(errors, source));
    }

    let compiler = ChunkCompiler::new(&syntax_tree, resolver);

    compiler
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

    // pub fn compile(&self, sources: &[(&str, &str)]) -> Result<Program, DustError<'_>> {}
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

    /// Number of registers that are used by local variables.
    locals: Vec<Local>,

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
            locals: Vec::new(),
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
        let register_count = self.get_next_register() + 1;

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
                    acc.max(instruction.destination().index)
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
            Constant::String(string_index) => string_index,
        };

        Address::constant(index)
    }

    fn combine_constants(
        &mut self,
        left: Constant,
        right: Constant,
        operation: SyntaxKind,
    ) -> Result<Constant, CompileError> {
        let combined = match (left, right) {
            (Constant::Byte(left), Constant::Byte(right)) => {
                let combined = match operation {
                    SyntaxKind::AdditionExpression => left.saturating_add(right),
                    SyntaxKind::SubtractionExpression => left.saturating_sub(right),
                    SyntaxKind::MultiplicationExpression => left.saturating_mul(right),
                    SyntaxKind::DivisionExpression => left.saturating_div(right),
                    SyntaxKind::ModuloExpression => left % right,
                    _ => todo!(),
                };

                Constant::Byte(combined)
            }
            (Constant::Float(left), Constant::Float(right)) => {
                let combined = match operation {
                    SyntaxKind::AdditionExpression => left + right,
                    SyntaxKind::SubtractionExpression => left - right,
                    SyntaxKind::MultiplicationExpression => left * right,
                    SyntaxKind::DivisionExpression => left / right,
                    SyntaxKind::ModuloExpression => left % right,
                    _ => todo!(),
                };

                Constant::Float(combined)
            }
            (Constant::Integer(left), Constant::Integer(right)) => {
                let combined = match operation {
                    SyntaxKind::AdditionExpression => left.saturating_add(right),
                    SyntaxKind::SubtractionExpression => left.saturating_sub(right),
                    SyntaxKind::MultiplicationExpression => left.saturating_mul(right),
                    SyntaxKind::DivisionExpression => left.saturating_div(right),
                    SyntaxKind::ModuloExpression => left % right,
                    _ => todo!(),
                };

                Constant::Integer(combined)
            }
            (Constant::String(left_index), Constant::String(right_index)) => {
                let concatenated = self
                    .resolver
                    .constants
                    .concatenate_strings(left_index, right_index);

                Constant::String(concatenated)
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
            SyntaxKind::LetStatement => self.compile_let_statement(&node),
            SyntaxKind::FunctionStatement => todo!("Compile function statement"),
            SyntaxKind::ExpressionStatement => todo!("Compile expression statement"),
            SyntaxKind::SemicolonStatement => todo!("Compile semicolon statement"),
            _ => Err(CompileError::ExpectedStatement {
                node_kind: node.kind,
                position: node.position,
            }),
        }
    }

    fn compile_main_function_statement(&mut self, node: &SyntaxNode) -> Result<(), CompileError> {
        let (start_children, child_count) = (node.payload.0 as usize, node.payload.1 as usize);
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
                } else if child_node.kind.is_statement() {
                    self.compile_statement(child_id)?;
                } else {
                    let return_emission = self.compile_expression(child_id, child_node)?;
                    let (return_operand, return_type) = match return_emission {
                        Emission::Instruction(instruction) => {
                            (self.handle_operand(instruction), instruction.operand_type())
                        }
                        Emission::Constant(constant) => {
                            let r#type = constant.operand_type();
                            let address = self.get_constant_address(constant);
                            let destination = Address::register(self.get_next_register());
                            let instruction =
                                Instruction::load(destination, address, r#type, false);
                            let return_operand = self.handle_operand(instruction);

                            (return_operand, r#type)
                        }
                    };

                    match return_type {
                        OperandType::BOOLEAN => self.return_type = TypeId::BOOLEAN,
                        OperandType::BYTE => self.return_type = TypeId::BYTE,
                        OperandType::CHARACTER => self.return_type = TypeId::CHARACTER,
                        OperandType::FLOAT => self.return_type = TypeId::FLOAT,
                        OperandType::INTEGER => self.return_type = TypeId::INTEGER,
                        OperandType::STRING => self.return_type = TypeId::STRING,
                        _ => todo!(),
                    }

                    let return_instruction = Instruction::r#return(
                        return_type != OperandType::NONE,
                        return_operand,
                        return_type,
                    );

                    self.instructions.push(return_instruction);
                }
            } else {
                self.compile_statement(child_id)?;
            }
        }

        Ok(())
    }

    fn compile_let_statement(&mut self, node: &SyntaxNode) -> Result<(), CompileError> {
        let declaration_id = DeclarationId(node.payload.0);
        let expression_id = SyntaxId(node.payload.1);

        let declaration = self
            .resolver
            .get_declaration_from_id(declaration_id)
            .copied()
            .ok_or(CompileError::MissingDeclaration { id: declaration_id })?;
        let expression_node = self
            .syntax_tree
            .get_node(expression_id)
            .ok_or(CompileError::MissingSyntaxNode { id: expression_id })?;
        let expression_emission = self.compile_expression(expression_id, expression_node)?;
        let destination_register = match expression_emission {
            Emission::Instruction(instruction) => {
                let destination_register = instruction.destination().index;

                self.instructions.push(instruction);

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
        let local = Local {
            declaration_id,
            register: destination_register,
            r#type: declaration.r#type,
        };

        self.locals.push(local);

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
            | SyntaxKind::ModuloExpression => self.compile_binary_expression(node_id, node),
            SyntaxKind::GroupedExpression => self.compile_grouped_expression(node_id, node),
            SyntaxKind::PathExpression => self.parse_path_expression(node_id, node),
            _ => Err(CompileError::ExpectedExpression {
                node_kind: node.kind,
                position: node.position,
            }),
        }
    }

    fn compile_boolean_expression(&mut self, node: &SyntaxNode) -> Result<Emission, CompileError> {
        Ok(Emission::Constant(Constant::Boolean(node.payload.0 != 0)))
    }

    fn compile_byte_expression(&mut self, node: &SyntaxNode) -> Result<Emission, CompileError> {
        Ok(Emission::Constant(Constant::Byte(node.payload.0 as u8)))
    }

    fn compile_character_expression(
        &mut self,
        _node_id: SyntaxId,
        node: &SyntaxNode,
    ) -> Result<Emission, CompileError> {
        let character = char::from_u32(node.payload.0).unwrap_or_default();

        Ok(Emission::Constant(Constant::Character(character)))
    }

    fn compile_float_expression(&mut self, node: &SyntaxNode) -> Result<Emission, CompileError> {
        let float = SyntaxNode::decode_float(node.payload);

        Ok(Emission::Constant(Constant::Float(float)))
    }

    fn compile_integer_expression(&mut self, node: &SyntaxNode) -> Result<Emission, CompileError> {
        let integer = SyntaxNode::decode_integer(node.payload);

        Ok(Emission::Constant(Constant::Integer(integer)))
    }

    fn compile_string_expression(&mut self, node: &SyntaxNode) -> Result<Emission, CompileError> {
        let string_index = node.payload.0 as u16;

        Ok(Emission::Constant(Constant::String(string_index)))
    }

    fn compile_binary_expression(
        &mut self,
        _node_id: SyntaxId,
        node: &SyntaxNode,
    ) -> Result<Emission, CompileError> {
        let left_index = SyntaxId(node.payload.0);
        let left = self.syntax_tree.nodes.get(left_index.0 as usize).ok_or(
            CompileError::MissingChild {
                parent_kind: node.kind,
                child_index: node.payload.0,
            },
        )?;
        let right_index = SyntaxId(node.payload.1);
        let right = self.syntax_tree.nodes.get(right_index.0 as usize).ok_or(
            CompileError::MissingChild {
                parent_kind: node.kind,
                child_index: node.payload.1,
            },
        )?;

        let left_emission = self.compile_expression(left_index, left)?;
        let right_emission = self.compile_expression(right_index, right)?;
        let (left_instruction, right_instruction) = match (left_emission, right_emission) {
            (Emission::Instruction(left), Emission::Instruction(right)) => (left, right),
            (Emission::Constant(left_value), Emission::Constant(right_value)) => {
                let combined = self.combine_constants(left_value, right_value, node.kind)?;

                return Ok(Emission::Constant(combined));
            }
            (Emission::Instruction(left_instruction), Emission::Constant(constant)) => {
                let r#type = constant.operand_type();
                let right_address = self.get_constant_address(constant);
                let destination = Address::register(self.get_next_register());
                let right_instruction =
                    Instruction::load(destination, right_address, r#type, false);

                (left_instruction, right_instruction)
            }
            (Emission::Constant(constant), Emission::Instruction(right_instruction)) => {
                let r#type = constant.operand_type();
                let left_address = self.get_constant_address(constant);
                let destination = Address::register(self.get_next_register());
                let left_instruction = Instruction::load(destination, left_address, r#type, false);

                (left_instruction, right_instruction)
            }
        };

        let left_address = self.handle_operand(left_instruction);
        let right_address = self.handle_operand(right_instruction);
        let combined_type = match (
            left_instruction.operand_type(),
            right_instruction.operand_type(),
        ) {
            (OperandType::CHARACTER, OperandType::CHARACTER) => OperandType::STRING,
            (left_type, _) => left_type,
        };
        let destination = Address::register(self.get_next_register());
        let instruction = match node.kind {
            SyntaxKind::AdditionExpression => {
                Instruction::add(destination, left_address, right_address, combined_type)
            }
            SyntaxKind::SubtractionExpression => {
                Instruction::subtract(destination, left_address, right_address, combined_type)
            }
            SyntaxKind::MultiplicationExpression => {
                Instruction::multiply(destination, left_address, right_address, combined_type)
            }
            SyntaxKind::DivisionExpression => {
                Instruction::divide(destination, left_address, right_address, combined_type)
            }
            SyntaxKind::ModuloExpression => {
                Instruction::modulo(destination, left_address, right_address, combined_type)
            }
            _ => unreachable!("Expected binary expression, found {}", node.kind),
        };

        Ok(Emission::Instruction(instruction))
    }

    fn compile_grouped_expression(
        &mut self,
        node_id: SyntaxId,
        node: &SyntaxNode,
    ) -> Result<Emission, CompileError> {
        let child_id = SyntaxId(node.payload.0);
        let child_node = self
            .syntax_tree
            .get_node(child_id)
            .ok_or(CompileError::MissingChild {
                parent_kind: node.kind,
                child_index: node.payload.0,
            })?;

        self.compile_expression(node_id, child_node)
    }

    fn parse_path_expression(
        &mut self,
        _node_id: SyntaxId,
        _node: &SyntaxNode,
    ) -> Result<Emission, CompileError> {
        todo!()
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

pub enum Emission {
    Instruction(Instruction),
    Constant(Constant),
}

pub enum Constant {
    Boolean(bool),
    Byte(u8),
    Character(char),
    Float(f64),
    Integer(i64),
    String(u16),
}

impl Constant {
    fn operand_type(&self) -> OperandType {
        match self {
            Constant::Boolean(_) => OperandType::BOOLEAN,
            Constant::Byte(_) => OperandType::BYTE,
            Constant::Character(_) => OperandType::CHARACTER,
            Constant::Float(_) => OperandType::FLOAT,
            Constant::Integer(_) => OperandType::INTEGER,
            Constant::String(_) => OperandType::STRING,
        }
    }
}
