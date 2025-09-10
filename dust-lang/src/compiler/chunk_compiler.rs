use std::{cell::RefCell, collections::HashMap, rc::Rc};

use rustc_hash::FxBuildHasher;
use tracing::{Level, debug, info, span};

use crate::{
    Address, Chunk, CompileError, ConstantTable, FunctionType, Instruction, OperandType, Operation,
    Resolver, Type,
    resolver::{DeclarationId, DeclarationKind, ScopeId, ScopeKind, TypeId, TypeNode},
    syntax_tree::{SyntaxId, SyntaxKind, SyntaxNode, SyntaxTree},
};

#[derive(Debug)]
pub struct ChunkCompiler<'a> {
    /// Target syntax tree for compilation.
    syntax_tree: Rc<SyntaxTree>,

    /// Target source code for compilation.
    source: &'a str,

    /// Constant collection from parsing that is pre-filled with strings. Other constant types are
    /// added during compilation after constant expression folding is applied.
    constants: ConstantTable,

    /// Context for modules, types and declarations provided by the parser.
    resolver: &'a Resolver,

    /// Global list of function prototypes that is filled during compilation.
    prototypes: Rc<RefCell<Vec<Chunk>>>,

    /// Bytecode instruction collection that is filled during compilation.
    instructions: Vec<Instruction>,

    /// Concatenated list of arguments referenced by CALL instructions.
    call_arguments: Vec<(Address, OperandType)>,

    /// Concatenated list of register indexes that are referenced by DROP instructions.
    drop_lists: Vec<u16>,

    /// Apparent return type of the function being compiled. This field is modified during compilation
    /// to reflect the actual return type of the function
    return_type: TypeId,

    /// Local variables declared in the function being compiled.
    locals: HashMap<DeclarationId, Local, FxBuildHasher>,

    /// Lowest register index after registers have been allocated for function arguments.
    minimum_register: u16,

    /// Index of the the chunk being compiled in the program's prototype list. For the main function,
    /// this is 0 as a default but the main chunk is actually the last one in the list.
    prototype_index: u16,
}

impl<'a> ChunkCompiler<'a> {
    pub fn new(
        syntax_tree: SyntaxTree,
        source: &'a str,
        resolver: &'a Resolver,
        prototypes: Rc<RefCell<Vec<Chunk>>>,
    ) -> Self {
        Self {
            syntax_tree: Rc::new(syntax_tree),
            source,
            constants: ConstantTable::new(),
            resolver,
            prototypes,
            instructions: Vec::new(),
            call_arguments: Vec::new(),
            drop_lists: Vec::new(),
            return_type: TypeId::NONE,
            locals: HashMap::default(),
            minimum_register: 0,
            prototype_index: 0,
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

        self.constants.trim_string_pool();

        Ok(Chunk {
            name: Some("main".to_string()),
            r#type: FunctionType::new([], [], return_type),
            instructions: self.instructions,
            constants: self.constants,
            call_arguments: self.call_arguments,
            drop_lists: self.drop_lists,
            register_count,
            prototype_index: 0,
            is_recursive: false,
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
            Constant::Character(character) => self.constants.add_character(character),
            Constant::Float(float) => self.constants.add_float(float),
            Constant::Integer(integer) => self.constants.add_integer(integer),
            Constant::String {
                pool_start,
                pool_end,
            } => self.constants.add_pooled_string(pool_start, pool_end),
            Constant::Function {
                prototype_index, ..
            } => prototype_index,
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

                    let combined = self.constants.push_str_to_string_pool(&string);

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
                    .constants
                    .get_string_pool(left_pool_start as usize..left_pool_end as usize);
                let right = self
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

                        let combined = self.constants.push_str_to_string_pool(&string);

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
                    .constants
                    .get_string_pool(pool_start as usize..pool_end as usize);
                let mut string = String::with_capacity(1 + right.len());

                string.push(left);
                string.push_str(right);

                let combined = match operation {
                    SyntaxKind::AdditionExpression => {
                        self.constants.push_str_to_string_pool(&string)
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
                    .constants
                    .get_string_pool(pool_start as usize..pool_end as usize);
                let mut string = String::with_capacity(left.len() + 1);

                string.push_str(left);
                string.push(right);

                let combined = match operation {
                    SyntaxKind::AdditionExpression => {
                        self.constants.push_str_to_string_pool(&string)
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
            SyntaxKind::FunctionStatement => self.compile_function_statement(&node),
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

        if child_count == 0 {
            let return_instruction =
                Instruction::r#return(false, Address::default(), OperandType::NONE);

            self.instructions.push(return_instruction);

            return Ok(());
        }

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
                let child_node = *self
                    .syntax_tree
                    .get_node(child_id)
                    .ok_or(CompileError::MissingSyntaxNode { id: child_id })?;

                self.compile_implicit_return(child_id, &child_node)?;
            } else {
                self.compile_statement(child_id)?;
            }
        }

        Ok(())
    }

    fn compile_expression_statement(&mut self, node: &SyntaxNode) -> Result<(), CompileError> {
        info!("Compiling expression statement");

        let expression_id = SyntaxId(node.children.0);
        let expression_node = *self
            .syntax_tree
            .get_node(expression_id)
            .ok_or(CompileError::MissingSyntaxNode { id: expression_id })?;
        let expression_emission = self.compile_expression(&expression_node)?;

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
            Emission::None => {}
        }

        Ok(())
    }

    fn compile_let_statement(&mut self, node: &SyntaxNode) -> Result<(), CompileError> {
        info!("Compiling let statement");

        let declaration_id = DeclarationId(node.payload);
        let expression_statement_id = SyntaxId(node.children.1);

        let declaration = *self
            .resolver
            .get_declaration(declaration_id)
            .ok_or(CompileError::MissingDeclaration { id: declaration_id })?;
        let is_mutable = declaration.kind == DeclarationKind::LocalMutable;
        let expression_statement_node = self.syntax_tree.get_node(expression_statement_id).ok_or(
            CompileError::MissingSyntaxNode {
                id: expression_statement_id,
            },
        )?;
        let expression_id = SyntaxId(expression_statement_node.children.0);
        let expression_node = *self
            .syntax_tree
            .get_node(expression_id)
            .ok_or(CompileError::MissingSyntaxNode { id: expression_id })?;
        let expression_emission = self.compile_expression(&expression_node)?;
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
            Emission::None => {
                return Err(CompileError::ExpectedExpression {
                    node_kind: expression_node.kind,
                    position: expression_node.position,
                });
            }
        };

        self.locals.insert(
            declaration_id,
            Local {
                location: destination_register,
                is_mutable,
            },
        );

        Ok(())
    }

    fn compile_function_statement(&mut self, node: &SyntaxNode) -> Result<(), CompileError> {
        info!("Compiling function statement");

        let declaration_id = DeclarationId(node.children.0);
        let function_expression_id = SyntaxId(node.children.1);
        let function_node = *self.syntax_tree.get_node(function_expression_id).ok_or(
            CompileError::MissingSyntaxNode {
                id: function_expression_id,
            },
        )?;

        let Emission::Constant(Constant::Function {
            prototype_index, ..
        }) = self.compile_function_expression(&function_node, declaration_id)?
        else {
            return Err(CompileError::ExpectedFunction {
                node_kind: function_node.kind,
                position: function_node.position,
            });
        };

        self.locals.insert(
            declaration_id,
            Local {
                location: prototype_index,
                is_mutable: false,
            },
        );

        Ok(())
    }

    fn compile_expression(&mut self, node: &SyntaxNode) -> Result<Emission, CompileError> {
        match node.kind {
            SyntaxKind::BooleanExpression => self.compile_boolean_expression(node),
            SyntaxKind::ByteExpression => self.compile_byte_expression(node),
            SyntaxKind::CharacterExpression => self.compile_character_expression(node),
            SyntaxKind::FloatExpression => self.compile_float_expression(node),
            SyntaxKind::IntegerExpression => self.compile_integer_expression(node),
            SyntaxKind::StringExpression => self.compile_string_expression(node),
            SyntaxKind::PathExpression => self.compile_path_expression(node),
            SyntaxKind::AdditionExpression
            | SyntaxKind::SubtractionExpression
            | SyntaxKind::MultiplicationExpression
            | SyntaxKind::DivisionExpression
            | SyntaxKind::ModuloExpression
            | SyntaxKind::AdditionAssignmentExpression
            | SyntaxKind::SubtractionAssignmentExpression
            | SyntaxKind::MultiplicationAssignmentExpression
            | SyntaxKind::DivisionAssignmentExpression
            | SyntaxKind::ModuloAssignmentExpression => self.compile_math_expression(node),
            SyntaxKind::GreaterThanExpression
            | SyntaxKind::GreaterThanOrEqualExpression
            | SyntaxKind::LessThanExpression
            | SyntaxKind::LessThanOrEqualExpression
            | SyntaxKind::EqualExpression
            | SyntaxKind::NotEqualExpression => self.compile_comparison_expression(node),
            SyntaxKind::AndExpression | SyntaxKind::OrExpression => {
                self.compile_logical_expression(node)
            }
            SyntaxKind::NotExpression | SyntaxKind::NegationExpression => {
                self.compile_unary_expression(node)
            }
            SyntaxKind::GroupedExpression => self.compile_grouped_expression(node),
            SyntaxKind::BlockExpression => self.compile_block_expression(node),
            SyntaxKind::WhileExpression => self.compile_while_expression(node),
            SyntaxKind::FunctionExpression => {
                self.compile_function_expression(node, DeclarationId::ANONYMOUS)
            }
            SyntaxKind::CallExpression => self.compile_call_expression(node),
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

        let string_start = node.position.0 + 1;
        let string_end = node.position.1 - 1;
        let string = &self.source[string_start as usize..string_end as usize];
        let (pool_start, pool_end) = self.constants.push_str_to_string_pool(string);

        Ok(Emission::Constant(Constant::String {
            pool_start,
            pool_end,
        }))
    }

    fn compile_math_expression(&mut self, node: &SyntaxNode) -> Result<Emission, CompileError> {
        info!("Compiling math expression");

        let left_index = SyntaxId(node.children.0);
        let left = *self.syntax_tree.nodes.get(left_index.0 as usize).ok_or(
            CompileError::MissingChild {
                parent_kind: node.kind,
                child_index: node.children.0,
            },
        )?;
        let right_index = SyntaxId(node.children.1);
        let right = *self.syntax_tree.nodes.get(right_index.0 as usize).ok_or(
            CompileError::MissingChild {
                parent_kind: node.kind,
                child_index: node.children.1,
            },
        )?;

        let left_emission = self.compile_expression(&left)?;
        let right_emission = self.compile_expression(&right)?;

        if let (Emission::Constant(left_value), Emission::Constant(right_value)) =
            (&left_emission, &right_emission)
        {
            let combined = self.combine_constants(*left_value, *right_value, node.kind)?;

            return Ok(Emission::Constant(combined));
        }

        let instructions_count_before = self.instructions.len();
        let (left_address, _) = left_emission.handle_as_operand(self);
        let (right_address, _) = right_emission.handle_as_operand(self);

        let destination = Address::register(self.get_next_register());
        let instruction = match node.kind {
            SyntaxKind::AdditionExpression => {
                let operand_type = self
                    .resolver
                    .resolve_type(TypeId(node.payload))
                    .ok_or(CompileError::MissingType {
                        type_id: TypeId(node.payload),
                    })?
                    .as_operand_type();

                Instruction::add(destination, left_address, right_address, operand_type)
            }
            SyntaxKind::AdditionAssignmentExpression => {
                let operand_type = self
                    .resolver
                    .resolve_type(TypeId(left.payload))
                    .ok_or(CompileError::MissingType {
                        type_id: TypeId(node.payload),
                    })?
                    .as_operand_type();

                self.instructions.truncate(instructions_count_before);

                Instruction::add(left_address, left_address, right_address, operand_type)
            }
            SyntaxKind::SubtractionExpression => {
                let operand_type = self
                    .resolver
                    .resolve_type(TypeId(node.payload))
                    .ok_or(CompileError::MissingType {
                        type_id: TypeId(node.payload),
                    })?
                    .as_operand_type();

                Instruction::subtract(destination, left_address, right_address, operand_type)
            }
            SyntaxKind::SubtractionAssignmentExpression => {
                let operand_type = self
                    .resolver
                    .resolve_type(TypeId(left.payload))
                    .ok_or(CompileError::MissingType {
                        type_id: TypeId(node.payload),
                    })?
                    .as_operand_type();

                self.instructions.truncate(instructions_count_before);

                Instruction::subtract(left_address, left_address, right_address, operand_type)
            }
            SyntaxKind::MultiplicationExpression => {
                let operand_type = self
                    .resolver
                    .resolve_type(TypeId(node.payload))
                    .ok_or(CompileError::MissingType {
                        type_id: TypeId(node.payload),
                    })?
                    .as_operand_type();

                Instruction::multiply(destination, left_address, right_address, operand_type)
            }
            SyntaxKind::MultiplicationAssignmentExpression => {
                let operand_type = self
                    .resolver
                    .resolve_type(TypeId(left.payload))
                    .ok_or(CompileError::MissingType {
                        type_id: TypeId(node.payload),
                    })?
                    .as_operand_type();

                self.instructions.truncate(instructions_count_before);

                Instruction::multiply(left_address, left_address, right_address, operand_type)
            }
            SyntaxKind::DivisionExpression => {
                let operand_type = self
                    .resolver
                    .resolve_type(TypeId(node.payload))
                    .ok_or(CompileError::MissingType {
                        type_id: TypeId(node.payload),
                    })?
                    .as_operand_type();

                Instruction::divide(destination, left_address, right_address, operand_type)
            }
            SyntaxKind::DivisionAssignmentExpression => {
                let operand_type = self
                    .resolver
                    .resolve_type(TypeId(left.payload))
                    .ok_or(CompileError::MissingType {
                        type_id: TypeId(node.payload),
                    })?
                    .as_operand_type();

                self.instructions.truncate(instructions_count_before);

                Instruction::divide(left_address, left_address, right_address, operand_type)
            }
            SyntaxKind::ModuloExpression => {
                let operand_type = self
                    .resolver
                    .resolve_type(TypeId(node.payload))
                    .ok_or(CompileError::MissingType {
                        type_id: TypeId(node.payload),
                    })?
                    .as_operand_type();

                Instruction::modulo(destination, left_address, right_address, operand_type)
            }
            SyntaxKind::ModuloAssignmentExpression => {
                let operand_type = self
                    .resolver
                    .resolve_type(TypeId(left.payload))
                    .ok_or(CompileError::MissingType {
                        type_id: TypeId(node.payload),
                    })?
                    .as_operand_type();

                self.instructions.truncate(instructions_count_before);

                Instruction::modulo(left_address, left_address, right_address, operand_type)
            }
            _ => unreachable!("Expected binary expression, found {}", node.kind),
        };

        Ok(Emission::Instruction(instruction, TypeId(node.payload)))
    }

    fn compile_comparison_expression(
        &mut self,
        node: &SyntaxNode,
    ) -> Result<Emission, CompileError> {
        let left_index = SyntaxId(node.children.0);
        let left = *self.syntax_tree.nodes.get(left_index.0 as usize).ok_or(
            CompileError::MissingChild {
                parent_kind: node.kind,
                child_index: node.children.0,
            },
        )?;
        let right_index = SyntaxId(node.children.1);
        let right = *self.syntax_tree.nodes.get(right_index.0 as usize).ok_or(
            CompileError::MissingChild {
                parent_kind: node.kind,
                child_index: node.children.1,
            },
        )?;

        let left_emission = self.compile_expression(&left)?;
        let right_emission = self.compile_expression(&right)?;

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
            Emission::None => {
                return Err(CompileError::ExpectedExpression {
                    node_kind: left.kind,
                    position: left.position,
                });
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
            Emission::None => {
                return Err(CompileError::ExpectedExpression {
                    node_kind: right.kind,
                    position: right.position,
                });
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

    fn compile_logical_expression(&mut self, node: &SyntaxNode) -> Result<Emission, CompileError> {
        info!("Compiling logical expression");

        let left_index = SyntaxId(node.children.0);
        let left = *self.syntax_tree.nodes.get(left_index.0 as usize).ok_or(
            CompileError::MissingChild {
                parent_kind: node.kind,
                child_index: node.children.0,
            },
        )?;
        let right_index = SyntaxId(node.children.1);
        let right = *self.syntax_tree.nodes.get(right_index.0 as usize).ok_or(
            CompileError::MissingChild {
                parent_kind: node.kind,
                child_index: node.children.1,
            },
        )?;

        let left_emission = self.compile_expression(&left)?;
        let right_emission = self.compile_expression(&right)?;

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
            Emission::None => {
                return Err(CompileError::ExpectedExpression {
                    node_kind: left.kind,
                    position: left.position,
                });
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
            Emission::None => {
                return Err(CompileError::ExpectedExpression {
                    node_kind: right.kind,
                    position: right.position,
                });
            }
        };

        Ok(emission)
    }

    fn compile_unary_expression(&mut self, node: &SyntaxNode) -> Result<Emission, CompileError> {
        info!("Compiling unary expression");

        let child_id = SyntaxId(node.children.0);
        let child_node =
            *self
                .syntax_tree
                .get_node(child_id)
                .ok_or(CompileError::MissingChild {
                    parent_kind: node.kind,
                    child_index: node.children.0,
                })?;
        let child_emission = self.compile_expression(&child_node)?;

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

        let (child_address, operand_type) = child_emission.handle_as_operand(self);
        let destination = Address::register(self.get_next_register());
        let negate_instruction = Instruction::negate(destination, child_address, operand_type);

        Ok(Emission::Instruction(
            negate_instruction,
            TypeId(node.payload),
        ))
    }

    fn compile_grouped_expression(&mut self, node: &SyntaxNode) -> Result<Emission, CompileError> {
        info!("Compiling grouped expression");

        let child_id = SyntaxId(node.children.0);
        let child_node =
            *self
                .syntax_tree
                .get_node(child_id)
                .ok_or(CompileError::MissingChild {
                    parent_kind: node.kind,
                    child_index: node.children.0,
                })?;

        self.compile_expression(&child_node)
    }

    fn compile_block_expression(&mut self, node: &SyntaxNode) -> Result<Emission, CompileError> {
        info!("Compiling block expression");

        let (start_children, child_count) = (node.children.0 as usize, node.children.1 as usize);

        if child_count == 0 {
            return Ok(Emission::None);
        }

        let end_children = start_children + child_count - 1;
        let mut current_child_index = start_children;

        while current_child_index < end_children {
            let child_id = *self.syntax_tree.children.get(current_child_index).ok_or(
                CompileError::MissingChild {
                    parent_kind: node.kind,
                    child_index: current_child_index as u32,
                },
            )?;
            current_child_index += 1;

            self.compile_statement(child_id)?;
        }

        let last_child_id =
            *self
                .syntax_tree
                .children
                .get(end_children)
                .ok_or(CompileError::MissingChild {
                    parent_kind: node.kind,
                    child_index: end_children as u32,
                })?;
        let last_child_node = *self
            .syntax_tree
            .get_node(last_child_id)
            .ok_or(CompileError::MissingSyntaxNode { id: last_child_id })?;
        let outer_scope_id = ScopeId(node.payload);
        let outer_scope = self
            .resolver
            .get_scope(outer_scope_id)
            .ok_or(CompileError::MissingScope { id: outer_scope_id })?;

        if last_child_node.kind.is_statement() {
            self.compile_statement(last_child_id)?;

            Ok(Emission::None)
        } else if outer_scope.kind == ScopeKind::Function && outer_scope_id != ScopeId::MAIN {
            self.compile_implicit_return(last_child_id, &last_child_node)?;

            Ok(Emission::None)
        } else {
            self.compile_expression(&last_child_node)
        }
    }

    fn compile_path_expression(&mut self, node: &SyntaxNode) -> Result<Emission, CompileError> {
        info!("Compiling path expression");

        let declaration_id = DeclarationId(node.children.0);
        let destination_register = self
            .locals
            .get(&declaration_id)
            .ok_or(CompileError::MissingLocal { declaration_id })?
            .location;
        let r#type = self
            .resolver
            .resolve_type(TypeId(node.payload))
            .ok_or(CompileError::MissingDeclaration { id: declaration_id })?
            .as_operand_type();

        if r#type == OperandType::FUNCTION {
            let prototype_index = self
                .locals
                .get(&declaration_id)
                .ok_or(CompileError::MissingLocal { declaration_id })?
                .location;

            return Ok(Emission::Constant(Constant::Function {
                type_id: TypeId(node.payload),
                prototype_index,
            }));
        }

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

    fn compile_while_expression(&mut self, node: &SyntaxNode) -> Result<Emission, CompileError> {
        info!("Compiling while expression");

        let condition_id = SyntaxId(node.children.0);
        let body_id = SyntaxId(node.children.1);

        let condition_node =
            *self
                .syntax_tree
                .get_node(condition_id)
                .ok_or(CompileError::MissingChild {
                    parent_kind: node.kind,
                    child_index: node.children.0,
                })?;
        let body_node = *self
            .syntax_tree
            .get_node(body_id)
            .ok_or(CompileError::MissingChild {
                parent_kind: node.kind,
                child_index: node.children.1,
            })?;
        let condition_emission = self.compile_expression(&condition_node)?;

        match condition_emission {
            Emission::Instruction(instruction, _) => {
                self.handle_operand(instruction);
            }
            Emission::Instructions(instructions, _) => {
                let comparison_or_test_instruction = instructions[0];

                self.instructions.push(comparison_or_test_instruction);
            }
            Emission::Constant(constant) => {
                let r#type = constant.operand_type();
                let address = self.get_constant_address(constant);
                let destination = Address::register(self.get_next_register());
                let load_instruction = Instruction::load(destination, address, r#type, false);

                self.handle_operand(load_instruction);
            }
            Emission::None => {
                return Err(CompileError::ExpectedExpression {
                    node_kind: condition_node.kind,
                    position: condition_node.position,
                });
            }
        }

        let jump_forward_index = self.instructions.len();

        self.instructions.push(Instruction::no_op());
        self.compile_expression_statement(&body_node)?;

        let jump_distance = (self.instructions.len() - jump_forward_index) as u16;

        let jump_forward_instruction = Instruction::jump(jump_distance, true);
        let jump_back_instruction = Instruction::jump(jump_distance, false);

        self.instructions[jump_forward_index] = jump_forward_instruction;
        self.instructions.push(jump_back_instruction);

        Ok(Emission::None)
    }

    fn compile_function_expression(
        &mut self,
        node: &SyntaxNode,
        declaration_id: DeclarationId,
    ) -> Result<Emission, CompileError> {
        info!("Compiling function expression");

        let function_signature_id = SyntaxId(node.children.0);
        let block_id = SyntaxId(node.children.1);
        let function_signature_node = *self.syntax_tree.get_node(function_signature_id).ok_or(
            CompileError::MissingSyntaxNode {
                id: function_signature_id,
            },
        )?;

        debug_assert_eq!(function_signature_node.kind, SyntaxKind::FunctionSignature);

        let value_parameters_node_id = function_signature_node.children.0;
        let value_parameters_node = *self
            .syntax_tree
            .get_node(SyntaxId(value_parameters_node_id))
            .ok_or(CompileError::MissingSyntaxNode {
                id: SyntaxId(value_parameters_node_id),
            })?;

        debug_assert_eq!(
            value_parameters_node.kind,
            SyntaxKind::FunctionValueParameters
        );

        let block_node = *self
            .syntax_tree
            .get_node(block_id)
            .ok_or(CompileError::MissingSyntaxNode { id: block_id })?;
        let function_type_id = TypeId(node.payload);
        let Some(TypeNode::Function {
            type_parameters: _,
            value_parameters,
            return_type,
        }) = self.resolver.get_type_node(function_type_id)
        else {
            return Err(CompileError::MissingType {
                type_id: function_type_id,
            });
        };
        let Some(Type::Function(function_type)) = self.resolver.resolve_type(function_type_id)
        else {
            return Err(CompileError::MissingType {
                type_id: function_type_id,
            });
        };
        let Some(value_parameter_nodes) = self
            .syntax_tree
            .get_children(
                value_parameters_node.children.0,
                value_parameters_node.children.1,
            )
            .map(|children| children.to_vec())
        else {
            return Err(CompileError::MissingChild {
                parent_kind: function_signature_node.kind,
                child_index: value_parameters.0,
            });
        };

        let mut function_compiler = ChunkCompiler {
            syntax_tree: self.syntax_tree.clone(),
            source: self.source,
            constants: ConstantTable::new(),
            resolver: self.resolver,
            prototypes: self.prototypes.clone(),
            instructions: Vec::new(),
            call_arguments: Vec::new(),
            drop_lists: Vec::new(),
            return_type: *return_type,
            locals: HashMap::default(),
            minimum_register: 0,
            prototype_index: self.prototypes.borrow().len() as u16,
        };

        for syntax_id in value_parameter_nodes {
            let parameter_node = *self
                .syntax_tree
                .get_node(syntax_id)
                .ok_or(CompileError::MissingSyntaxNode { id: syntax_id })?;
            let parameter_declaration_id = DeclarationId(parameter_node.payload);
            let register = function_compiler.get_next_register();
            function_compiler.minimum_register += 1;

            function_compiler.locals.insert(
                parameter_declaration_id,
                Local {
                    location: register,
                    is_mutable: false,
                },
            );
        }

        let name = if declaration_id == DeclarationId::ANONYMOUS {
            None
        } else {
            let declaration = self
                .resolver
                .get_declaration(declaration_id)
                .ok_or(CompileError::MissingDeclaration { id: declaration_id })?;
            let name_range = declaration.identifier_position.as_usize_range();
            let name = self.source[name_range].to_string();

            Some(name)
        };

        let _ = function_compiler.compile_block_expression(&block_node)?;

        let function_chunk = Chunk {
            name,
            r#type: *function_type,
            register_count: function_compiler.get_next_register(),
            instructions: function_compiler.instructions,
            constants: function_compiler.constants,
            call_arguments: function_compiler.call_arguments,
            drop_lists: function_compiler.drop_lists,
            prototype_index: function_compiler.prototype_index,
            is_recursive: false,
        };

        self.prototypes.borrow_mut().push(function_chunk);

        Ok(Emission::Constant(Constant::Function {
            type_id: function_type_id,
            prototype_index: function_compiler.prototype_index,
        }))
    }

    fn compile_call_expression(&mut self, node: &SyntaxNode) -> Result<Emission, CompileError> {
        info!("Compiling call expression");

        let function_node_id = SyntaxId(node.children.0);
        let arguments_node_id = SyntaxId(node.children.1);

        let function_node =
            *self
                .syntax_tree
                .get_node(function_node_id)
                .ok_or(CompileError::MissingChild {
                    parent_kind: node.kind,
                    child_index: node.children.0,
                })?;
        let arguments_node =
            *self
                .syntax_tree
                .get_node(arguments_node_id)
                .ok_or(CompileError::MissingChild {
                    parent_kind: node.kind,
                    child_index: node.children.1,
                })?;

        let Emission::Constant(Constant::Function {
            type_id,
            prototype_index,
        }) = self.compile_expression(&function_node)?
        else {
            return Err(CompileError::ExpectedFunction {
                node_kind: function_node.kind,
                position: function_node.position,
            });
        };

        let children = self
            .syntax_tree
            .get_children(arguments_node.children.0, arguments_node.children.1)
            .ok_or(CompileError::MissingChild {
                parent_kind: arguments_node.kind,
                child_index: arguments_node.children.0,
            })?
            .to_vec();

        let arguments_start_index = self.call_arguments.len() as u16;

        for child_id in children {
            let child_node = *self
                .syntax_tree
                .get_node(child_id)
                .ok_or(CompileError::MissingSyntaxNode { id: child_id })?;
            let argument_emission = self.compile_expression(&child_node)?;
            let argument_address = argument_emission.handle_as_operand(self);

            self.call_arguments.push(argument_address);
        }

        let destination = Address::register(self.get_next_register());
        let Type::Function(function_type) = self
            .resolver
            .resolve_type(type_id)
            .ok_or(CompileError::MissingType { type_id })?
        else {
            return Err(CompileError::ExpectedFunction {
                node_kind: function_node.kind,
                position: function_node.position,
            });
        };
        let operand_type = function_type.return_type.as_operand_type();
        let call_instruction = Instruction::call(
            destination,
            prototype_index,
            arguments_start_index,
            operand_type,
        );

        Ok(Emission::Instruction(
            call_instruction,
            TypeId(node.payload),
        ))
    }

    fn compile_implicit_return(
        &mut self,
        node_id: SyntaxId,
        node: &SyntaxNode,
    ) -> Result<(), CompileError> {
        if node.kind.is_item() {
            self.compile_item(node_id)?;

            let return_instruction =
                Instruction::r#return(false, Address::default(), OperandType::NONE);

            self.instructions.push(return_instruction);
        } else if node.kind.is_statement() {
            self.compile_statement(node_id)?;

            let return_instruction =
                Instruction::r#return(false, Address::default(), OperandType::NONE);

            self.instructions.push(return_instruction);
        } else {
            let return_emission = self.compile_expression(node)?;
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
                        Constant::Character(_) => (TypeId::CHARACTER, OperandType::CHARACTER),
                        Constant::Float(_) => (TypeId::FLOAT, OperandType::FLOAT),
                        Constant::Integer(_) => (TypeId::INTEGER, OperandType::INTEGER),
                        Constant::String { .. } => (TypeId::STRING, OperandType::STRING),
                        Constant::Function { type_id, .. } => (type_id, OperandType::FUNCTION),
                    };
                    let address = self.get_constant_address(constant);
                    let destination = Address::register(self.get_next_register());
                    let instruction = Instruction::load(destination, address, operand_type, false);
                    let return_operand = self.handle_operand(instruction);

                    (return_operand, r#type, operand_type)
                }
                Emission::None => (Address::default(), TypeId::NONE, OperandType::NONE),
            };

            self.return_type = return_type;
            let return_instruction = Instruction::r#return(
                return_operand_type != OperandType::NONE,
                return_operand,
                return_operand_type,
            );

            self.instructions.push(return_instruction);
        }
        Ok(())
    }
}

#[derive(Clone, Debug)]
pub enum Emission {
    Instruction(Instruction, TypeId),
    Instructions(Vec<Instruction>, TypeId),
    Constant(Constant),
    None,
}

impl Emission {
    fn handle_as_operand(self, compiler: &mut ChunkCompiler) -> (Address, OperandType) {
        match self {
            Emission::Instruction(instruction, _) => (
                compiler.handle_operand(instruction),
                instruction.operand_type(),
            ),
            Emission::Instructions(instructions, _) => {
                let first_instruction = instructions[0];
                let destination = first_instruction.destination();

                compiler.instructions.extend(instructions.iter());

                (destination, first_instruction.operand_type())
            }
            Emission::Constant(constant) => {
                let r#type = constant.operand_type();
                let address = compiler.get_constant_address(constant);
                let destination = Address::register(compiler.get_next_register());
                let load_instruction = Instruction::load(destination, address, r#type, false);

                (compiler.handle_operand(load_instruction), r#type)
            }
            Emission::None => (Address::default(), OperandType::NONE),
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
    String {
        pool_start: u32,
        pool_end: u32,
    },
    Function {
        type_id: TypeId,
        prototype_index: u16,
    },
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
            Constant::Function { .. } => OperandType::FUNCTION,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, PartialOrd, Ord)]
struct Local {
    location: u16,
    is_mutable: bool,
}
