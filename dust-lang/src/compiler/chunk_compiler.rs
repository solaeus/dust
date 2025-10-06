use std::collections::HashMap;

use rustc_hash::FxBuildHasher;
use smallvec::SmallVec;
use tracing::{Level, debug, info, span};

use crate::{
    chunk::Chunk,
    compiler::CompileContext,
    instruction::{Address, Instruction, OperandType, Operation},
    native_function::NativeFunction,
    resolver::{Declaration, DeclarationId, DeclarationKind, ScopeId, TypeId},
    source::{Position, SourceFileId},
    syntax_tree::{SyntaxId, SyntaxKind, SyntaxNode, SyntaxTree},
    r#type::{FunctionType, Type},
};

use super::CompileError;

#[derive(Debug)]
pub struct ChunkCompiler<'a> {
    declaration_id: DeclarationId,
    file_id: SourceFileId,

    function_type: FunctionType,

    context: &'a mut CompileContext,

    /// Local variables declared in the function being compiled.
    locals: HashMap<DeclarationId, Local, FxBuildHasher>,

    /// Lowest register index after registers have been allocated for function arguments.
    minimum_register: u16,

    /// Bytecode instruction collection that is filled during compilation.
    instructions: Vec<Instruction>,

    /// Concatenated list of arguments referenced by CALL instructions.
    call_arguments: Vec<(Address, OperandType)>,

    /// Concatenated list of register indexes that are referenced by DROP instructions.
    drop_lists: Vec<u16>,
}

impl<'a> ChunkCompiler<'a> {
    pub fn new(
        declaration_id: DeclarationId,
        file_id: SourceFileId,
        function_type: FunctionType,
        context: &'a mut CompileContext,
    ) -> Self {
        Self {
            declaration_id,
            file_id,
            function_type,
            context,
            instructions: Vec::new(),
            call_arguments: Vec::new(),
            drop_lists: Vec::new(),
            locals: HashMap::default(),
            minimum_register: 0,
        }
    }

    pub fn compile_main(mut self) -> Result<Chunk, CompileError> {
        let span = span!(Level::INFO, "main");
        let _enter = span.enter();

        let root_node = *self
            .syntax_tree()?
            .get_node(SyntaxId(0))
            .ok_or(CompileError::MissingSyntaxNode { id: SyntaxId(0) })?;

        self.compile_item(&root_node)?;
        self.finish()
    }

    pub fn finish(mut self) -> Result<Chunk, CompileError> {
        // self.context.constants.finalize_string_pool();

        let name_position = if matches!(
            self.declaration_id,
            DeclarationId::MAIN | DeclarationId::ANONYMOUS
        ) {
            None
        } else {
            let declaration = self
                .context
                .resolver
                .get_declaration(self.declaration_id)
                .ok_or(CompileError::MissingDeclaration {
                    declaration_id: self.declaration_id,
                })?;

            Some(declaration.position)
        };
        let register_count = self.get_next_register();

        Ok(Chunk {
            name_position,
            r#type: self.function_type,
            instructions: self.instructions,
            call_arguments: self.call_arguments,
            drop_lists: self.drop_lists,
            register_count,
        })
    }

    fn syntax_tree(&self) -> Result<&SyntaxTree, CompileError> {
        self.context.file_trees.get(self.file_id.0 as usize).ok_or(
            CompileError::MissingSyntaxTree {
                declaration_id: DeclarationId::ANONYMOUS,
            },
        )
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

    fn current_scope_id(&self) -> Result<ScopeId, CompileError> {
        let declaration = self
            .context
            .resolver
            .get_declaration(self.declaration_id)
            .ok_or(CompileError::MissingDeclaration {
                declaration_id: self.declaration_id,
            })?;

        Ok(declaration.scope_id)
    }

    fn get_constant_address(&mut self, constant: Constant) -> Address {
        let index = match constant {
            Constant::Boolean(boolean) => return Address::encoded(boolean as u16),
            Constant::Byte(byte) => return Address::encoded(byte as u16),
            Constant::Character(character) => self.context.constants.add_character(character),
            Constant::Float(float) => self.context.constants.add_float(float),
            Constant::Integer(integer) => self.context.constants.add_integer(integer),
            Constant::String {
                pool_start,
                pool_end,
            } => self
                .context
                .constants
                .add_pooled_string(pool_start, pool_end),
            Constant::Function {
                prototype_index, ..
            } => prototype_index,
            Constant::NativeFunction {
                native_function, ..
            } => native_function.index,
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

                    let combined = self
                        .context
                        .constants
                        .push_str_to_string_pool(string.as_bytes());

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
                    .context
                    .constants
                    .get_string_pool_range(left_pool_start as usize..left_pool_end as usize);
                let right = self
                    .context
                    .constants
                    .get_string_pool_range(right_pool_start as usize..right_pool_end as usize);

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

                        let combined = self
                            .context
                            .constants
                            .push_str_to_string_pool(string.as_bytes());

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
                    .context
                    .constants
                    .get_string_pool_range(pool_start as usize..pool_end as usize);
                let mut string = String::with_capacity(1 + right.len());

                string.push(left);
                string.push_str(right);

                let combined = match operation {
                    SyntaxKind::AdditionExpression => self
                        .context
                        .constants
                        .push_str_to_string_pool(string.as_bytes()),
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
                    .context
                    .constants
                    .get_string_pool_range(pool_start as usize..pool_end as usize);
                let mut string = String::with_capacity(left.len() + 1);

                string.push_str(left);
                string.push(right);

                let combined = match operation {
                    SyntaxKind::AdditionExpression => self
                        .context
                        .constants
                        .push_str_to_string_pool(string.as_bytes()),
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
        if let Operation::LOAD = instruction.operation() {
            instruction.b_address()
        } else {
            self.instructions.push(instruction);

            instruction.destination()
        }
    }

    fn compile_item(&mut self, node: &SyntaxNode) -> Result<(), CompileError> {
        match node.kind {
            SyntaxKind::MainFunctionItem => self.compile_main_function_statement(node),
            SyntaxKind::ModuleItem => self.compile_module_item(node),
            SyntaxKind::FunctionItem => self.compile_function_item(node),
            SyntaxKind::UseItem => self.compile_use_item(node),
            _ => Err(CompileError::ExpectedItem {
                node_kind: node.kind,
                position: Position::new(self.file_id, node.span),
            }),
        }
    }

    fn compile_statement(&mut self, node: &SyntaxNode) -> Result<(), CompileError> {
        match node.kind {
            SyntaxKind::ExpressionStatement => self.compile_expression_statement(node),
            SyntaxKind::LetStatement | SyntaxKind::LetMutStatement => {
                self.compile_let_statement(node)
            }
            SyntaxKind::ReassignStatement => self.compile_reassign_statement(node),
            SyntaxKind::SemicolonStatement => todo!("Compile semicolon statement"),
            _ => Err(CompileError::ExpectedStatement {
                node_kind: node.kind,
                position: Position::new(self.file_id, node.span),
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
            let child_id = *self
                .syntax_tree()?
                .children
                .get(current_child_index)
                .ok_or(CompileError::MissingChild {
                    parent_kind: node.kind,
                    child_index: current_child_index as u32,
                })?;
            let child_node = *self
                .syntax_tree()?
                .get_node(child_id)
                .ok_or(CompileError::MissingSyntaxNode { id: child_id })?;
            current_child_index += 1;

            if current_child_index == end_children {
                self.compile_implicit_return(&child_node)?;
            } else if child_node.kind.is_statement() {
                self.compile_statement(&child_node)?;
            } else if child_node.kind.is_item() {
                self.compile_item(&child_node)?;
            } else {
                return Err(CompileError::ExpectedStatement {
                    node_kind: child_node.kind,
                    position: Position::new(self.file_id, child_node.span),
                });
            }
        }

        Ok(())
    }

    fn compile_module_item(&mut self, node: &SyntaxNode) -> Result<(), CompileError> {
        info!("Compiling module item");

        let (start_children, child_count) = (node.children.0 as usize, node.children.1 as usize);
        let end_children = start_children + child_count;

        if child_count == 0 {
            return Ok(());
        }

        let mut current_child_index = start_children;

        while current_child_index < end_children {
            let child_id = *self
                .syntax_tree()?
                .children
                .get(current_child_index)
                .ok_or(CompileError::MissingChild {
                    parent_kind: node.kind,
                    child_index: current_child_index as u32,
                })?;
            let child_node = *self
                .syntax_tree()?
                .get_node(child_id)
                .ok_or(CompileError::MissingSyntaxNode { id: child_id })?;
            current_child_index += 1;

            self.compile_item(&child_node)?;
        }

        Ok(())
    }

    fn compile_use_item(&mut self, node: &SyntaxNode) -> Result<(), CompileError> {
        info!("Compiling use item");

        todo!()
    }

    fn compile_expression_statement(&mut self, node: &SyntaxNode) -> Result<(), CompileError> {
        info!("Compiling expression statement");

        let exression_id = SyntaxId(node.children.0);
        let expression_node = *self
            .syntax_tree()?
            .get_node(exression_id)
            .ok_or(CompileError::MissingSyntaxNode { id: exression_id })?;

        let _expression_emission = self.compile_expression(&expression_node)?;

        Ok(())
    }

    fn compile_let_statement(&mut self, node: &SyntaxNode) -> Result<(), CompileError> {
        info!("Compiling let statement");

        let path_id = SyntaxId(node.children.0);
        let expression_statement_id = SyntaxId(node.children.1);
        let load_destination = Address::register(self.get_next_register());

        let expression_statement = *self
            .syntax_tree()?
            .get_node(expression_statement_id)
            .ok_or(CompileError::MissingSyntaxNode {
                id: expression_statement_id,
            })?;
        let expression = *self
            .syntax_tree()?
            .get_node(SyntaxId(expression_statement.children.0))
            .ok_or(CompileError::MissingSyntaxNode {
                id: SyntaxId(expression_statement.children.0),
            })?;
        let expression_emission = self.compile_expression(&expression)?;
        let expression_type = expression_emission.r#type().clone();
        let type_id = self.context.resolver.register_type(&expression_type);
        let local_address = expression_emission.handle_as_operand(self);

        let path = *self
            .syntax_tree()?
            .get_node(path_id)
            .ok_or(CompileError::MissingSyntaxNode { id: path_id })?;
        let files = self.context.source.read_files();
        let source_file =
            files
                .get(self.file_id.0 as usize)
                .ok_or(CompileError::MissingSourceFile {
                    file_id: self.file_id,
                })?;
        let variable_name_bytes =
            &source_file.source_code.as_ref()[path.span.0 as usize..path.span.1 as usize];

        let declaration_kind = if node.kind == SyntaxKind::LetStatement {
            DeclarationKind::Local { shadowed: None }
        } else {
            DeclarationKind::LocalMutable { shadowed: None }
        };
        let declaration_id = self.context.resolver.add_declaration(
            variable_name_bytes,
            Declaration {
                kind: declaration_kind,
                scope_id: self.current_scope_id()?,
                type_id,
                position: Position::new(self.file_id, node.span),
                is_public: false,
            },
        );

        let load_instruction = Instruction::load(
            load_destination,
            local_address,
            expression_type.as_operand_type(),
            false,
        );

        self.instructions.push(load_instruction);
        self.locals.insert(
            declaration_id,
            Local {
                r#type: expression_type.clone(),
                address: load_destination,
            },
        );

        Ok(())
    }

    fn compile_reassign_statement(&mut self, node: &SyntaxNode) -> Result<(), CompileError> {
        info!("Compiling reassign statement");

        todo!()
    }

    fn compile_function_item(&mut self, node: &SyntaxNode) -> Result<(), CompileError> {
        info!("Compiling function statement");

        todo!()
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
                position: Position::new(self.file_id, node.span),
            }),
        }
    }

    fn compile_boolean_expression(&mut self, node: &SyntaxNode) -> Result<Emission, CompileError> {
        info!("Compiling boolean expression");

        Ok(Emission::Constant(
            Constant::Boolean(node.children.0 != 0),
            Type::Boolean,
        ))
    }

    fn compile_byte_expression(&mut self, node: &SyntaxNode) -> Result<Emission, CompileError> {
        info!("Compiling byte expression");

        Ok(Emission::Constant(
            Constant::Byte(node.children.0 as u8),
            Type::Byte,
        ))
    }

    fn compile_character_expression(
        &mut self,
        node: &SyntaxNode,
    ) -> Result<Emission, CompileError> {
        info!("Compiling character expression");

        let character = char::from_u32(node.children.0).unwrap_or_default();

        Ok(Emission::Constant(
            Constant::Character(character),
            Type::Character,
        ))
    }

    fn compile_float_expression(&mut self, node: &SyntaxNode) -> Result<Emission, CompileError> {
        info!("Compiling float expression");

        let float = SyntaxNode::decode_float(node.children);

        Ok(Emission::Constant(Constant::Float(float), Type::Float))
    }

    fn compile_integer_expression(&mut self, node: &SyntaxNode) -> Result<Emission, CompileError> {
        info!("Compiling integer expression");

        let integer = SyntaxNode::decode_integer(node.children);

        Ok(Emission::Constant(
            Constant::Integer(integer),
            Type::Integer,
        ))
    }

    fn compile_string_expression(&mut self, node: &SyntaxNode) -> Result<Emission, CompileError> {
        info!("Compiling string expression");

        let string_start = node.span.0 + 1;
        let string_end = node.span.1 - 1;
        let files = self.context.source.read_files();
        let source_file =
            files
                .get(self.file_id.0 as usize)
                .ok_or(CompileError::MissingSourceFile {
                    file_id: self.file_id,
                })?;
        let string = &source_file.source_code.as_ref()[string_start as usize..string_end as usize];
        let (pool_start, pool_end) = self.context.constants.push_str_to_string_pool(string);

        Ok(Emission::Constant(
            Constant::String {
                pool_start,
                pool_end,
            },
            Type::String,
        ))
    }

    fn compile_math_expression(&mut self, node: &SyntaxNode) -> Result<Emission, CompileError> {
        info!("Compiling math expression");

        let left_index = SyntaxId(node.children.0);
        let left = *self.syntax_tree()?.nodes.get(left_index.0 as usize).ok_or(
            CompileError::MissingChild {
                parent_kind: node.kind,
                child_index: node.children.0,
            },
        )?;
        let right_index = SyntaxId(node.children.1);
        let right = *self
            .syntax_tree()?
            .nodes
            .get(right_index.0 as usize)
            .ok_or(CompileError::MissingChild {
                parent_kind: node.kind,
                child_index: node.children.1,
            })?;

        let left_emission = self.compile_expression(&left)?;
        let right_emission = self.compile_expression(&right)?;

        if let (
            Emission::Constant(left_value, left_type),
            Emission::Constant(right_value, _right_type),
        ) = (&left_emission, &right_emission)
        {
            let combined = self.combine_constants(*left_value, *right_value, node.kind)?;
            let combined_type = if left_type == &Type::Character {
                Type::String
            } else {
                left_type.clone()
            };

            return Ok(Emission::Constant(combined, combined_type));
        }

        let left_type = left_emission.r#type();
        let _right_type = right_emission.r#type();

        let instructions_count_before = self.instructions.len();
        let r#type = if left_type == &Type::Character {
            Type::String
        } else {
            left_type.clone()
        };
        let left_address = left_emission.handle_as_operand(self);
        let right_address = right_emission.handle_as_operand(self);
        let destination = Address::register(self.get_next_register());
        let operand_type = r#type.as_operand_type();

        let instruction = match node.kind {
            SyntaxKind::AdditionExpression => {
                Instruction::add(destination, left_address, right_address, operand_type)
            }
            SyntaxKind::AdditionAssignmentExpression => {
                self.instructions.truncate(instructions_count_before);

                Instruction::add(left_address, left_address, right_address, operand_type)
            }
            SyntaxKind::SubtractionExpression => {
                Instruction::subtract(destination, left_address, right_address, operand_type)
            }
            SyntaxKind::SubtractionAssignmentExpression => {
                self.instructions.truncate(instructions_count_before);

                Instruction::subtract(left_address, left_address, right_address, operand_type)
            }
            SyntaxKind::MultiplicationExpression => {
                Instruction::multiply(destination, left_address, right_address, operand_type)
            }
            SyntaxKind::MultiplicationAssignmentExpression => {
                self.instructions.truncate(instructions_count_before);

                Instruction::multiply(left_address, left_address, right_address, operand_type)
            }
            SyntaxKind::DivisionExpression => {
                Instruction::divide(destination, left_address, right_address, operand_type)
            }
            SyntaxKind::DivisionAssignmentExpression => {
                self.instructions.truncate(instructions_count_before);

                Instruction::divide(left_address, left_address, right_address, operand_type)
            }
            SyntaxKind::ModuloExpression => {
                Instruction::modulo(destination, left_address, right_address, operand_type)
            }
            SyntaxKind::ModuloAssignmentExpression => {
                self.instructions.truncate(instructions_count_before);

                Instruction::modulo(left_address, left_address, right_address, operand_type)
            }
            _ => unreachable!("Expected binary expression, found {}", node.kind),
        };

        Ok(Emission::Instruction(instruction, r#type))
    }

    fn compile_comparison_expression(
        &mut self,
        node: &SyntaxNode,
    ) -> Result<Emission, CompileError> {
        let left_index = SyntaxId(node.children.0);
        let left = *self.syntax_tree()?.nodes.get(left_index.0 as usize).ok_or(
            CompileError::MissingChild {
                parent_kind: node.kind,
                child_index: node.children.0,
            },
        )?;
        let right_index = SyntaxId(node.children.1);
        let right = *self
            .syntax_tree()?
            .nodes
            .get(right_index.0 as usize)
            .ok_or(CompileError::MissingChild {
                parent_kind: node.kind,
                child_index: node.children.1,
            })?;

        let left_emission = self.compile_expression(&left)?;
        let right_emission = self.compile_expression(&right)?;

        if let (
            Emission::Constant(left_value, _left_type),
            Emission::Constant(right_value, _right_type),
        ) = (&left_emission, &right_emission)
        {
            let combined = self.combine_constants(*left_value, *right_value, node.kind)?;

            return Ok(Emission::Constant(combined, Type::Boolean));
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
            Emission::Constant(constant, type_id) => {
                let operand_type = type_id.as_operand_type();
                let address = self.get_constant_address(*constant);
                let load_instruction = Instruction::load(destination, address, operand_type, false);

                (self.handle_operand(load_instruction), operand_type)
            }
            Emission::None => {
                return Err(CompileError::ExpectedExpression {
                    node_kind: left.kind,
                    position: Position::new(self.file_id, left.span),
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
            Emission::Constant(constant, type_id) => {
                let r#type = type_id.as_operand_type();
                let address = self.get_constant_address(constant);
                let load_instruction = Instruction::load(destination, address, r#type, false);

                (self.handle_operand(load_instruction), r#type)
            }
            Emission::None => {
                return Err(CompileError::ExpectedExpression {
                    node_kind: right.kind,
                    position: Position::new(self.file_id, right.span),
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
            Type::Boolean,
        ))
    }

    fn compile_logical_expression(&mut self, node: &SyntaxNode) -> Result<Emission, CompileError> {
        info!("Compiling logical expression");

        let left_index = SyntaxId(node.children.0);
        let left = *self.syntax_tree()?.nodes.get(left_index.0 as usize).ok_or(
            CompileError::MissingChild {
                parent_kind: node.kind,
                child_index: node.children.0,
            },
        )?;
        let right_index = SyntaxId(node.children.1);
        let right = *self
            .syntax_tree()?
            .nodes
            .get(right_index.0 as usize)
            .ok_or(CompileError::MissingChild {
                parent_kind: node.kind,
                child_index: node.children.1,
            })?;

        let left_emission = self.compile_expression(&left)?;
        let right_emission = self.compile_expression(&right)?;

        if let (
            Emission::Constant(left_value, left_type),
            Emission::Constant(right_value, _right_type),
        ) = (&left_emission, &right_emission)
        {
            let combined = self.combine_constants(*left_value, *right_value, node.kind)?;

            return Ok(Emission::Constant(combined, left_type.clone()));
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
            Emission::Constant(constant, type_id) => {
                let operand_type = type_id.as_operand_type();
                let address = self.get_constant_address(*constant);
                let destination = Address::register(self.get_next_register());
                let load_instruction = Instruction::load(destination, address, operand_type, false);

                self.handle_operand(load_instruction)
            }
            Emission::None => {
                return Err(CompileError::ExpectedExpression {
                    node_kind: left.kind,
                    position: Position::new(self.file_id, left.span),
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

                Emission::Instruction(instruction, Type::Boolean)
            }
            Emission::Instructions(instructions, r#type) => {
                Emission::Instructions(instructions, r#type)
            }
            Emission::Constant(constant, type_id) => Emission::Constant(constant, type_id),
            Emission::None => {
                return Err(CompileError::ExpectedExpression {
                    node_kind: right.kind,
                    position: Position::new(self.file_id, right.span),
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
                .syntax_tree()?
                .get_node(child_id)
                .ok_or(CompileError::MissingChild {
                    parent_kind: node.kind,
                    child_index: node.children.0,
                })?;
        let child_emission = self.compile_expression(&child_node)?;

        if let Emission::Constant(child_value, child_type) = &child_emission {
            let evaluated = match (node.kind, child_value) {
                (SyntaxKind::NotExpression, Constant::Boolean(value)) => Constant::Boolean(!value),
                (SyntaxKind::NegationExpression, Constant::Integer(value)) => {
                    Constant::Integer(-value)
                }
                (SyntaxKind::NegationExpression, Constant::Float(value)) => Constant::Float(-value),
                _ => todo!("Error"),
            };

            return Ok(Emission::Constant(evaluated, child_type.clone()));
        }

        let r#type = child_emission.r#type().clone();
        let child_address = child_emission.handle_as_operand(self);
        let destination = Address::register(self.get_next_register());
        let negate_instruction =
            Instruction::negate(destination, child_address, r#type.as_operand_type());

        Ok(Emission::Instruction(negate_instruction, r#type))
    }

    fn compile_grouped_expression(&mut self, node: &SyntaxNode) -> Result<Emission, CompileError> {
        info!("Compiling grouped expression");

        let child_id = SyntaxId(node.children.0);
        let child_node =
            *self
                .syntax_tree()?
                .get_node(child_id)
                .ok_or(CompileError::MissingChild {
                    parent_kind: node.kind,
                    child_index: node.children.0,
                })?;

        self.compile_expression(&child_node)
    }

    fn compile_block_expression(&mut self, node: &SyntaxNode) -> Result<Emission, CompileError> {
        info!("Compiling block expression");

        let start_children = node.children.0 as usize;
        let child_count = node.children.1 as usize;

        if child_count == 0 {
            return Ok(Emission::None);
        }

        let end_children = start_children + child_count - 1;
        let mut current_child_index = start_children;

        while current_child_index < end_children {
            let child_id = *self
                .syntax_tree()?
                .children
                .get(current_child_index)
                .ok_or(CompileError::MissingChild {
                    parent_kind: node.kind,
                    child_index: current_child_index as u32,
                })?;
            let child_node = *self
                .syntax_tree()?
                .get_node(child_id)
                .ok_or(CompileError::MissingSyntaxNode { id: child_id })?;
            current_child_index += 1;

            self.compile_statement(&child_node)?;
        }

        let last_child_id =
            *self
                .syntax_tree()?
                .children
                .get(end_children)
                .ok_or(CompileError::MissingChild {
                    parent_kind: node.kind,
                    child_index: end_children as u32,
                })?;
        let last_child_node = *self
            .syntax_tree()?
            .get_node(last_child_id)
            .ok_or(CompileError::MissingSyntaxNode { id: last_child_id })?;

        if last_child_node.kind.is_expression() {
            self.compile_expression(&last_child_node)
        } else {
            self.compile_statement(&last_child_node)?;

            Ok(Emission::None)
        }
    }

    fn compile_path_expression(&mut self, node: &SyntaxNode) -> Result<Emission, CompileError> {
        info!("Compiling path expression");

        let files = self.context.source.read_files();
        let source_file =
            files
                .get(self.file_id.0 as usize)
                .ok_or(CompileError::MissingSourceFile {
                    file_id: self.file_id,
                })?;
        let variable_name_bytes =
            &source_file.source_code.as_ref()[node.span.0 as usize..node.span.1 as usize];

        match variable_name_bytes {
            b"read_line" => {
                let read_line_function = NativeFunction::from_str("read_line").unwrap();

                return Ok(Emission::Constant(
                    Constant::NativeFunction {
                        native_function: read_line_function,
                    },
                    Type::Function(Box::new(read_line_function.r#type())),
                ));
            }
            b"write_line" => {
                let write_line_function = NativeFunction::from_str("write_line").unwrap();

                return Ok(Emission::Constant(
                    Constant::NativeFunction {
                        native_function: write_line_function,
                    },
                    Type::Function(Box::new(write_line_function.r#type())),
                ));
            }
            _ => {}
        }

        let (declaration_id, declaration) = self
            .context
            .resolver
            .find_declaration_in_scope(variable_name_bytes, self.current_scope_id()?)
            .ok_or(CompileError::UndeclaredVariable {
                name: unsafe { String::from_utf8_unchecked(variable_name_bytes.to_vec()) },
                position: Position::new(self.file_id, node.span),
            })?;
        let local =
            self.locals
                .get(&declaration_id)
                .cloned()
                .ok_or(CompileError::UndeclaredVariable {
                    name: unsafe { String::from_utf8_unchecked(variable_name_bytes.to_vec()) },
                    position: Position::new(self.file_id, node.span),
                })?;

        drop(files);

        let load_instruction = Instruction::load(
            Address::register(self.get_next_register()),
            local.address,
            declaration.type_id.as_operand_type(),
            false,
        );

        Ok(Emission::Instruction(load_instruction, local.r#type))
    }

    fn compile_while_expression(&mut self, node: &SyntaxNode) -> Result<Emission, CompileError> {
        info!("Compiling while expression");

        let condition_id = SyntaxId(node.children.0);
        let body_id = SyntaxId(node.children.1);

        let condition_node =
            *self
                .syntax_tree()?
                .get_node(condition_id)
                .ok_or(CompileError::MissingChild {
                    parent_kind: node.kind,
                    child_index: node.children.0,
                })?;
        let body_node =
            *self
                .syntax_tree()?
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
            Emission::Constant(constant, type_id) => {
                let operand_type = type_id.as_operand_type();
                let address = self.get_constant_address(constant);
                let destination = Address::register(self.get_next_register());
                let load_instruction = Instruction::load(destination, address, operand_type, false);

                self.handle_operand(load_instruction);
            }
            Emission::None => {
                return Err(CompileError::ExpectedExpression {
                    node_kind: condition_node.kind,
                    position: Position::new(self.file_id, condition_node.span),
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

        let function_signature_node = *self.syntax_tree()?.get_node(function_signature_id).ok_or(
            CompileError::MissingSyntaxNode {
                id: function_signature_id,
            },
        )?;

        debug_assert_eq!(function_signature_node.kind, SyntaxKind::FunctionSignature);

        let value_parameters_node_id = function_signature_node.children.0;
        let value_parameters_node = *self
            .syntax_tree()?
            .get_node(SyntaxId(value_parameters_node_id))
            .ok_or(CompileError::MissingSyntaxNode {
                id: SyntaxId(value_parameters_node_id),
            })?;

        debug_assert_eq!(
            value_parameters_node.kind,
            SyntaxKind::FunctionValueParameters
        );

        let body_node = *self
            .syntax_tree()?
            .get_node(block_id)
            .ok_or(CompileError::MissingSyntaxNode { id: block_id })?;
        let Some(value_parameter_nodes) = self
            .syntax_tree()?
            .get_children(
                value_parameters_node.children.0,
                value_parameters_node.children.1,
            )
            .map(|children| children.to_vec())
        else {
            return Err(CompileError::ChildIndexOutOfBounds {
                parent_kind: function_signature_node.kind,
                children_start: value_parameters_node.children.0,
                child_count: value_parameters_node.children.1,
            });
        };

        let mut value_parameter_types =
            SmallVec::<[TypeId; 4]>::with_capacity(value_parameter_nodes.len());

        let function_type = todo!();

        let span = span!(Level::INFO, "function");
        let _enter = span.enter();

        let mut function_compiler =
            ChunkCompiler::new(declaration_id, self.file_id, function_type, self.context);

        function_compiler.compile_implicit_return(&body_node)?;

        let function_chunk = function_compiler.finish()?;
        let prototype_index = self.context.prototypes.len() as u16;

        self.context
            .prototypes
            .insert(declaration_id, function_chunk);

        Ok(Emission::Constant(
            Constant::Function { prototype_index },
            Type::Function(Box::new(function_type)),
        ))
    }

    fn compile_call_expression(&mut self, node: &SyntaxNode) -> Result<Emission, CompileError> {
        fn handle_call_arguments(
            compiler: &mut ChunkCompiler,
            arguments_node: &SyntaxNode,
        ) -> Result<(), CompileError> {
            debug_assert_eq!(arguments_node.kind, SyntaxKind::CallValueArguments);

            let children = compiler
                .syntax_tree()?
                .get_children(arguments_node.children.0, arguments_node.children.1)
                .ok_or(CompileError::MissingChild {
                    parent_kind: arguments_node.kind,
                    child_index: arguments_node.children.0,
                })?
                .to_vec();

            for child_id in children {
                let child_node = *compiler
                    .syntax_tree()?
                    .get_node(child_id)
                    .ok_or(CompileError::MissingSyntaxNode { id: child_id })?;
                let argument_emission = compiler.compile_expression(&child_node)?;
                let operand_type = argument_emission.r#type().as_operand_type();
                let argument_address = argument_emission.handle_as_operand(compiler);

                compiler
                    .call_arguments
                    .push((argument_address, operand_type));
            }

            Ok(())
        }

        info!("Compiling call expression");

        let function_node_id = SyntaxId(node.children.0);
        let arguments_node_id = SyntaxId(node.children.1);

        let function_node =
            *self
                .syntax_tree()?
                .get_node(function_node_id)
                .ok_or(CompileError::MissingChild {
                    parent_kind: node.kind,
                    child_index: node.children.0,
                })?;
        let arguments_node =
            *self
                .syntax_tree()?
                .get_node(arguments_node_id)
                .ok_or(CompileError::MissingChild {
                    parent_kind: node.kind,
                    child_index: node.children.1,
                })?;

        debug_assert_eq!(arguments_node.kind, SyntaxKind::CallValueArguments);

        let expression_emission = self.compile_expression(&function_node)?;

        let Emission::Constant(Constant::Function { prototype_index }, r#type) =
            expression_emission
        else {
            if let Emission::Constant(Constant::NativeFunction { native_function }, r#type) =
                expression_emission
            {
                let destination = Address::register(self.get_next_register());
                let call_arguments_start_index = self.call_arguments.len() as u16;

                handle_call_arguments(self, &arguments_node)?;

                let call_native_instruction = Instruction::call_native(
                    destination,
                    native_function,
                    call_arguments_start_index,
                );

                return Ok(Emission::Instruction(call_native_instruction, r#type));
            }

            return Err(CompileError::ExpectedFunction {
                node_kind: function_node.kind,
                position: Position::new(self.file_id, function_node.span),
            });
        };

        let arguments_start_index = self.call_arguments.len() as u16;

        handle_call_arguments(self, &arguments_node)?;

        let destination_index = self.get_next_register();
        let Type::Function(function_type) = r#type else {
            return Err(CompileError::ExpectedFunction {
                node_kind: function_node.kind,
                position: Position::new(self.file_id, function_node.span),
            });
        };
        let r#type = function_type.return_type.clone();
        let argument_count = self.call_arguments.len() as u16;
        let call_instruction = Instruction::call(
            destination_index,
            prototype_index,
            arguments_start_index,
            argument_count,
            r#type.as_operand_type(),
        );

        Ok(Emission::Instruction(call_instruction, r#type))
    }

    fn compile_implicit_return(&mut self, node: &SyntaxNode) -> Result<(), CompileError> {
        let return_instruction = if node.kind.is_item() {
            self.compile_item(node)?;

            Instruction::r#return(false, Address::default(), OperandType::NONE)
        } else if node.kind.is_statement() {
            self.compile_statement(node)?;

            Instruction::r#return(false, Address::default(), OperandType::NONE)
        } else {
            let return_emission = self.compile_expression(node)?;
            let (return_operand, return_type) = match return_emission {
                Emission::Instruction(instruction, r#type) => {
                    let mut return_operand = self.handle_operand(instruction);

                    if let Some(last_instruction) = self.instructions.last()
                        && last_instruction.operation() == Operation::LOAD
                        && last_instruction.destination() == return_operand
                    {
                        return_operand = last_instruction.b_address();

                        self.instructions.pop();
                    }

                    (return_operand, r#type)
                }
                Emission::Instructions(instructions, r#type) => {
                    let last_instruction = instructions.last().unwrap();

                    self.instructions.extend(instructions.iter());

                    (last_instruction.destination(), r#type)
                }
                Emission::Constant(constant, r#type) => {
                    let operand_type = r#type.as_operand_type();
                    let address = self.get_constant_address(constant);
                    let destination = Address::register(self.get_next_register());
                    let instruction = Instruction::load(destination, address, operand_type, false);
                    let return_operand = self.handle_operand(instruction);

                    (return_operand, r#type)
                }
                Emission::None => (Address::default(), Type::None),
            };

            self.function_type.return_type = return_type.clone();
            let return_operand_type = return_type.as_operand_type();

            Instruction::r#return(
                return_operand_type != OperandType::NONE,
                return_operand,
                return_operand_type,
            )
        };

        if self
            .instructions
            .last()
            .is_some_and(|instruction| instruction.operation() == Operation::RETURN)
        {
            return Ok(());
        }

        self.instructions.push(return_instruction);

        Ok(())
    }
}

#[derive(Clone, Debug)]
pub enum Emission {
    Instruction(Instruction, Type),
    Instructions(Vec<Instruction>, Type),
    Constant(Constant, Type),
    None,
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
            Emission::Constant(constant, type_id) => {
                let address = compiler.get_constant_address(constant);
                let destination = Address::register(compiler.get_next_register());
                let load_instruction =
                    Instruction::load(destination, address, type_id.as_operand_type(), false);

                compiler.handle_operand(load_instruction)
            }
            Emission::None => Address::default(),
        }
    }

    fn r#type(&self) -> &Type {
        match self {
            Emission::Instruction(_, r#type) => r#type,
            Emission::Instructions(_, r#type) => r#type,
            Emission::Constant(_, r#type) => r#type,
            Emission::None => &Type::None,
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
    Function { prototype_index: u16 },
    NativeFunction { native_function: NativeFunction },
}

#[derive(Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
struct Local {
    address: Address,
    r#type: Type,
}
