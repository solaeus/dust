use std::collections::HashMap;

use rustc_hash::FxBuildHasher;
use smallvec::{SmallVec, smallvec};
use tracing::{debug, info, span};

use crate::{
    chunk::Chunk,
    compiler::{CompileContext, binder::Binder},
    instruction::{Address, Instruction, MemoryKind, OperandType, Operation},
    native_function::NativeFunction,
    resolver::{Declaration, DeclarationId, DeclarationKind, Scope, ScopeId, ScopeKind},
    source::{Position, SourceFileId},
    syntax_tree::{SyntaxId, SyntaxKind, SyntaxNode, SyntaxTree},
    r#type::{FunctionType, Type},
};

use super::CompileError;

#[derive(Debug)]
pub struct ChunkCompiler<'a> {
    declaration_id: Option<DeclarationId>,

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

    current_scope_id: ScopeId,
}

impl<'a> ChunkCompiler<'a> {
    pub fn new(
        declaration_id: Option<DeclarationId>,
        file_id: SourceFileId,
        function_type: FunctionType,
        context: &'a mut CompileContext,
        starting_scope_id: ScopeId,
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
            current_scope_id: starting_scope_id,
        }
    }

    pub fn compile_main(mut self) -> Result<Chunk, CompileError> {
        let root_node =
            *self
                .syntax_tree()?
                .get_node(SyntaxId(0))
                .ok_or(CompileError::MissingSyntaxNode {
                    syntax_id: SyntaxId(0),
                })?;

        self.compile_item(&root_node)?;
        self.finish()
    }

    pub fn finish(mut self) -> Result<Chunk, CompileError> {
        // self.context.constants.finalize_string_pool();

        let name_position = if let Some(declaration_id) = self.declaration_id {
            let declaration = self
                .context
                .resolver
                .get_declaration(declaration_id)
                .ok_or(CompileError::MissingDeclaration { declaration_id })?;

            Some(declaration.position)
        } else {
            None
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
                file_id: self.file_id,
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
                SyntaxKind::GreaterThanExpression => Constant::Boolean(left || right),
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
                SyntaxKind::ExponentExpression => Constant::Float(left.powf(right)),
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
                SyntaxKind::ExponentExpression => {
                    Constant::Integer(left.saturating_pow(right as u32))
                }
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
        if let Operation::MOVE = instruction.operation() {
            instruction.b_address()
        } else {
            self.instructions.push(instruction);

            instruction.destination()
        }
    }

    fn compile_item(&mut self, node: &SyntaxNode) -> Result<(), CompileError> {
        match node.kind {
            SyntaxKind::MainFunctionItem => self.compile_main_function_item(node),
            SyntaxKind::ModuleItem => self.compile_module_item(node),
            SyntaxKind::FunctionItem => self.compile_function_item(node),
            SyntaxKind::UseItem => self.compile_use_item(node),
            _ => Err(CompileError::ExpectedItem {
                node_kind: node.kind,
                position: Position::new(self.file_id, node.span),
            }),
        }
    }

    fn compile_statement(&mut self, node: &SyntaxNode) -> Result<Emission, CompileError> {
        match node.kind {
            SyntaxKind::ExpressionStatement => self.compile_expression_statement(node),
            SyntaxKind::LetStatement | SyntaxKind::LetMutStatement => {
                self.compile_let_statement(node)
            }
            _ => Err(CompileError::ExpectedStatement {
                node_kind: node.kind,
                position: Position::new(self.file_id, node.span),
            }),
        }
    }

    fn compile_main_function_item(&mut self, node: &SyntaxNode) -> Result<(), CompileError> {
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
            let child_node =
                *self
                    .syntax_tree()?
                    .get_node(child_id)
                    .ok_or(CompileError::MissingSyntaxNode {
                        syntax_id: child_id,
                    })?;
            current_child_index += 1;

            if current_child_index == end_children {
                self.compile_implicit_return(&child_node)?;
            } else if child_node.kind.is_statement() {
                let statement_emission = self.compile_statement(&child_node)?;

                match statement_emission {
                    Emission::Instruction(instruction, _) => {
                        self.instructions.push(instruction);
                    }
                    Emission::Instructions(instructions, _) => {
                        self.instructions.extend(instructions);
                    }
                    Emission::Constant(_, _)
                    | Emission::Local(_)
                    | Emission::Function(_, _)
                    | Emission::None => {}
                }
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
            let child_node =
                *self
                    .syntax_tree()?
                    .get_node(child_id)
                    .ok_or(CompileError::MissingSyntaxNode {
                        syntax_id: child_id,
                    })?;
            current_child_index += 1;

            self.compile_item(&child_node)?;
        }

        Ok(())
    }

    fn compile_use_item(&mut self, node: &SyntaxNode) -> Result<(), CompileError> {
        info!("Compiling use item");

        let syntax_tree = self.syntax_tree()?;

        let path_id = SyntaxId(node.children.0);
        let path_node = syntax_tree
            .get_node(path_id)
            .ok_or(CompileError::MissingSyntaxNode { syntax_id: path_id })?;
        let path_segments_node_ids = syntax_tree
            .get_children(path_node.children.0, path_node.children.1)
            .ok_or(CompileError::MissingChildren {
                parent_kind: path_node.kind,
                start_index: path_node.children.0,
                count: path_node.children.1,
            })?;
        let path_segments_nodes: SmallVec<[_; 4]> = path_segments_node_ids
            .iter()
            .map(|id| {
                syntax_tree
                    .get_node(*id)
                    .ok_or(CompileError::MissingSyntaxNode { syntax_id: *id })
            })
            .collect::<Result<_, _>>()?;

        let files = self.context.source.read_files();
        let file = files
            .get(self.file_id.0 as usize)
            .ok_or(CompileError::MissingSourceFile {
                file_id: self.file_id,
            })?;

        let module_name_node = path_segments_nodes.first().unwrap();
        let module_name_bytes = &file.source_code.as_ref()
            [module_name_node.span.0 as usize..module_name_node.span.1 as usize];
        let module_name = unsafe { std::str::from_utf8_unchecked(module_name_bytes) };
        let (module_import_id, module_import) = self
            .context
            .resolver
            .find_declarations(module_name)
            .ok_or(CompileError::MissingDeclarations {
                name: module_name.to_string(),
            })?
            .into_iter()
            .find(|(_id, declaration)| matches!(declaration.kind, DeclarationKind::Module { .. }))
            .ok_or(CompileError::UndeclaredVariable {
                name: module_name.to_string(),
                position: Position::new(self.file_id, module_name_node.span),
            })?;

        let (final_declaration_id, final_declaration) = if path_segments_nodes.len() > 1 {
            let mut current_scope_id =
                if let DeclarationKind::Module { inner_scope_id, .. } = &module_import.kind {
                    *inner_scope_id
                } else {
                    unreachable!("Expected module declaration");
                };
            let mut current_declaration_id = module_import_id;
            let mut current_declaration = module_import;

            for segment_node in path_segments_nodes.iter().skip(1) {
                let segment_bytes = &file.source_code.as_ref()
                    [segment_node.span.0 as usize..segment_node.span.1 as usize];
                let segment_name = unsafe { std::str::from_utf8_unchecked(segment_bytes) };
                let (declaration_id, declaration) = self
                    .context
                    .resolver
                    .find_declaration_in_scope(segment_name, current_scope_id)
                    .ok_or(CompileError::UndeclaredVariable {
                        name: segment_name.to_string(),
                        position: Position::new(self.file_id, segment_node.span),
                    })?;

                current_scope_id = match &declaration.kind {
                    DeclarationKind::Module { inner_scope_id, .. } => *inner_scope_id,
                    DeclarationKind::Function { inner_scope_id, .. } => *inner_scope_id,
                    _ => {
                        return Err(CompileError::CannotImport {
                            name: segment_name.to_string(),
                            position: Position::new(self.file_id, segment_node.span),
                        });
                    }
                };
                current_declaration_id = declaration_id;
                current_declaration = declaration;
            }

            (current_declaration_id, current_declaration)
        } else {
            (module_import_id, module_import)
        };

        drop(path_segments_nodes);

        if let DeclarationKind::Function {
            inner_scope_id,
            syntax_id,
        } = final_declaration.kind
        {
            let function_type = self
                .context
                .resolver
                .resolve_type(final_declaration.type_id)
                .ok_or(CompileError::MissingType {
                    type_id: final_declaration.type_id,
                })?
                .into_function_type()
                .ok_or(CompileError::ExpectedFunctionType {
                    type_id: final_declaration.type_id,
                })?;

            drop(files);

            let function_node = *self
                .context
                .file_trees
                .get(final_declaration.position.file_id.0 as usize)
                .ok_or(CompileError::MissingSyntaxTree {
                    file_id: final_declaration.position.file_id,
                })?
                .get_node(syntax_id)
                .ok_or(CompileError::MissingSyntaxNode { syntax_id })?;

            let span = span!(tracing::Level::INFO, "bind_import_function");
            let _enter = span.enter();

            let mut binder = Binder::new(
                final_declaration.position.file_id,
                self.context.source.clone(),
                &mut self.context.resolver,
                self.context
                    .file_trees
                    .get(final_declaration.position.file_id.0 as usize)
                    .ok_or(CompileError::MissingSyntaxTree {
                        file_id: final_declaration.position.file_id,
                    })?,
                inner_scope_id,
            );

            binder.bind_function_item(syntax_id, &function_node)?;

            drop(_enter);

            match function_node.kind {
                SyntaxKind::PublicFunctionItem => {
                    let mut importer = ChunkCompiler::new(
                        Some(final_declaration_id),
                        final_declaration.position.file_id,
                        function_type,
                        self.context,
                        inner_scope_id,
                    );

                    importer.compile_function_item(&function_node)?;
                }
                _ => {
                    return Err(CompileError::ExpectedFunction {
                        node_kind: function_node.kind,
                        position: final_declaration.position,
                    });
                }
            }
        } else {
            todo!()
        }

        Ok(())
    }

    fn compile_expression_statement(
        &mut self,
        node: &SyntaxNode,
    ) -> Result<Emission, CompileError> {
        info!("Compiling expression statement");

        let exression_id = SyntaxId(node.children.0);
        let expression_node =
            *self
                .syntax_tree()?
                .get_node(exression_id)
                .ok_or(CompileError::MissingSyntaxNode {
                    syntax_id: exression_id,
                })?;

        let expression_emission = self.compile_expression(&expression_node)?;
        let statement_emission = match expression_emission {
            Emission::Instruction(instruction, _) => Emission::Instruction(instruction, Type::None),
            Emission::Instructions(instructions, _) => {
                Emission::Instructions(instructions, Type::None)
            }
            Emission::Constant(_, _)
            | Emission::Local(_)
            | Emission::Function(_, _)
            | Emission::None => Emission::None,
        };

        Ok(statement_emission)
    }

    fn compile_let_statement(&mut self, node: &SyntaxNode) -> Result<Emission, CompileError> {
        info!("Compiling let statement");

        let path_id = SyntaxId(node.children.0);
        let expression_statement_id = SyntaxId(node.children.1);
        let expression_statement = *self
            .syntax_tree()?
            .get_node(expression_statement_id)
            .ok_or(CompileError::MissingSyntaxNode {
                syntax_id: expression_statement_id,
            })?;
        let expression = *self
            .syntax_tree()?
            .get_node(SyntaxId(expression_statement.children.0))
            .ok_or(CompileError::MissingSyntaxNode {
                syntax_id: SyntaxId(expression_statement.children.0),
            })?;
        let expression_emission = self.compile_expression(&expression)?;
        let expression_type = expression_emission.r#type().clone();
        let type_id = self.context.resolver.add_type(&expression_type);
        let destination = self.get_next_register();
        let (local_address, statement_emission) = match expression_emission {
            Emission::Instruction(instruction, _) => {
                let mut instructions = SmallVec::<[Instruction; 8]>::new();
                let local_address = instruction.destination();
                let move_instruction = Instruction::r#move(
                    destination,
                    local_address,
                    expression_type.as_operand_type(),
                    false,
                );

                instructions.push(instruction);
                instructions.push(move_instruction);

                (
                    local_address,
                    Emission::Instructions(instructions, Type::None),
                )
            }
            Emission::Instructions(mut instructions, _) => {
                let local_address = instructions.last().unwrap().destination();
                let move_instruction = Instruction::r#move(
                    destination,
                    local_address,
                    expression_type.as_operand_type(),
                    false,
                );

                instructions.push(move_instruction);

                (
                    local_address,
                    Emission::Instructions(instructions, Type::None),
                )
            }
            Emission::Constant(constant, _) => {
                let constant_address = self.get_constant_address(constant);
                let move_instruction = Instruction::r#move(
                    destination,
                    constant_address,
                    expression_type.as_operand_type(),
                    false,
                );

                (
                    constant_address,
                    Emission::Instruction(move_instruction, Type::None),
                )
            }
            Emission::Local(Local { address, .. }) => {
                let move_instruction = Instruction::r#move(
                    destination,
                    address,
                    expression_type.as_operand_type(),
                    false,
                );

                (address, Emission::Instruction(move_instruction, Type::None))
            }
            Emission::Function(address, _) => (address, Emission::None),
            Emission::None => (Address::default(), Emission::None),
        };
        let path_node = *self
            .syntax_tree()?
            .get_node(path_id)
            .ok_or(CompileError::MissingSyntaxNode { syntax_id: path_id })?;
        let files = self.context.source.read_files();
        let source_file =
            files
                .get(self.file_id.0 as usize)
                .ok_or(CompileError::MissingSourceFile {
                    file_id: self.file_id,
                })?;
        let variable_name_bytes =
            &source_file.source_code.as_ref()[path_node.span.0 as usize..path_node.span.1 as usize];
        let variable_name = unsafe { str::from_utf8_unchecked(variable_name_bytes) };
        let shadowed = self
            .context
            .resolver
            .find_declaration_in_scope(variable_name, self.current_scope_id)
            .map(|(id, _)| id);
        let declaration_kind = if node.kind == SyntaxKind::LetStatement {
            DeclarationKind::Local { shadowed }
        } else {
            DeclarationKind::LocalMutable { shadowed }
        };
        let declaration_id = self.context.resolver.add_declaration(
            variable_name,
            Declaration {
                kind: declaration_kind,
                scope_id: self.current_scope_id,
                type_id,
                position: Position::new(self.file_id, node.span),
                is_public: false,
            },
        );

        drop(files);

        if local_address.memory == MemoryKind::REGISTER {
            self.locals.insert(
                declaration_id,
                Local {
                    r#type: expression_type,
                    address: local_address,
                },
            );
        } else {
            self.locals.insert(
                declaration_id,
                Local {
                    r#type: expression_type,
                    address: Address::register(destination),
                },
            );
        }

        Ok(statement_emission)
    }

    fn compile_function_item(&mut self, node: &SyntaxNode) -> Result<(), CompileError> {
        info!("Compiling function item");

        let path_id = SyntaxId(node.children.0);
        let path_node = *self
            .syntax_tree()?
            .get_node(path_id)
            .ok_or(CompileError::MissingSyntaxNode { syntax_id: path_id })?;

        let function_expression_id = SyntaxId(node.children.1);
        let function_expression_node = *self
            .syntax_tree()?
            .get_node(function_expression_id)
            .ok_or(CompileError::MissingSyntaxNode {
                syntax_id: function_expression_id,
            })?;

        let files = self.context.source.read_files();
        let source_file =
            files
                .get(self.file_id.0 as usize)
                .ok_or(CompileError::MissingSourceFile {
                    file_id: self.file_id,
                })?;

        let path_bytes =
            &source_file.source_code.as_ref()[path_node.span.0 as usize..path_node.span.1 as usize];
        let path_str = unsafe { str::from_utf8_unchecked(path_bytes) };
        let function_name = path_str.split("::").last().unwrap_or(path_str);

        let (declaration_id, declaration) = self
            .context
            .resolver
            .find_declaration_in_scope(function_name, self.current_scope_id)
            .ok_or(CompileError::UndeclaredVariable {
                name: function_name.to_string(),
                position: Position::new(self.file_id, path_node.span),
            })?;
        let r#type = self
            .context
            .resolver
            .resolve_type(declaration.type_id)
            .ok_or(CompileError::MissingType {
                type_id: declaration.type_id,
            })?;
        let function_type =
            r#type
                .clone()
                .into_function_type()
                .ok_or(CompileError::ExpectedFunctionType {
                    type_id: declaration.type_id,
                })?;

        drop(files);

        let Emission::Function(address, _) = self.compile_function_expression(
            &function_expression_node,
            Some((declaration_id, declaration)),
            Some(function_type),
        )?
        else {
            return Err(CompileError::ExpectedFunction {
                node_kind: function_expression_node.kind,
                position: Position::new(self.file_id, function_expression_node.span),
            });
        };
        let local = Local { address, r#type };

        self.locals.insert(declaration_id, local);

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
            SyntaxKind::ListExpression => self.compile_list_expression(node),
            SyntaxKind::IndexExpression => self.compile_index_expression(node),
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
            | SyntaxKind::ModuloAssignmentExpression
            | SyntaxKind::ExponentExpression => self.compile_math_expression(node),
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
            SyntaxKind::FunctionExpression => self.compile_function_expression(node, None, None),
            SyntaxKind::CallExpression => self.compile_call_expression(node),
            SyntaxKind::AsExpression => self.compile_as_expression(node),
            SyntaxKind::IfExpression => self.compile_if_expression(node),
            SyntaxKind::ElseExpression => self.compile_else_expression(node),
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

    fn compile_list_expression(&mut self, node: &SyntaxNode) -> Result<Emission, CompileError> {
        info!("Compiling list expression");

        let (start_children, child_count) = (node.children.0 as usize, node.children.1 as usize);
        let list_destination = self.get_next_register();
        let mut instructions = {
            let mut instructions = SmallVec::<[Instruction; 8]>::with_capacity(child_count + 1);
            let placeholder = Instruction::no_op();

            instructions.push(placeholder);

            instructions
        };
        let mut current_child_index = start_children;
        let mut element_type = None;

        for list_index in 0..child_count {
            let child_id = *self
                .syntax_tree()?
                .children
                .get(current_child_index)
                .ok_or(CompileError::MissingChild {
                    parent_kind: node.kind,
                    child_index: current_child_index as u32,
                })?;
            let child_node =
                *self
                    .syntax_tree()?
                    .get_node(child_id)
                    .ok_or(CompileError::MissingSyntaxNode {
                        syntax_id: child_id,
                    })?;
            current_child_index += 1;

            let element_emission = self.compile_expression(&child_node)?;
            let new_type = element_emission.r#type().clone();
            let element_address = element_emission.handle_as_operand(self);

            if let Some(existing_type) = &element_type {
                if existing_type != &new_type {
                    todo!("Error");
                }
            } else {
                element_type = Some(new_type);
            }

            let element_type = element_type.as_ref().unwrap();

            let set_list_instruction = Instruction::set_list(
                list_destination,
                element_address,
                list_index as u16,
                element_type.as_operand_type(),
            );

            instructions.push(set_list_instruction);
        }

        let element_type = element_type.unwrap_or(Type::None);
        let list_type = match &element_type {
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

        let new_list_instruction =
            Instruction::new_list(list_destination, child_count as u16, list_type);

        instructions[0] = new_list_instruction;

        Ok(Emission::Instructions(
            instructions,
            Type::List(Box::new(element_type)),
        ))
    }

    fn compile_index_expression(&mut self, node: &SyntaxNode) -> Result<Emission, CompileError> {
        info!("Compiling index expression");

        let list_index = SyntaxId(node.children.0);
        let list_node =
            *self
                .syntax_tree()?
                .get_node(list_index)
                .ok_or(CompileError::MissingSyntaxNode {
                    syntax_id: list_index,
                })?;
        let index_index = SyntaxId(node.children.1);
        let index_node =
            *self
                .syntax_tree()?
                .get_node(index_index)
                .ok_or(CompileError::MissingSyntaxNode {
                    syntax_id: index_index,
                })?;
        let list_emission = self.compile_expression(&list_node)?;
        let element_type = list_emission
            .r#type()
            .as_element_type()
            .ok_or(CompileError::ExpectedList {
                found_type: list_emission.r#type().clone(),
                position: Position::new(self.file_id, list_node.span),
            })?
            .clone();
        let index_emission = self.compile_expression(&index_node)?;
        let list_address = list_emission.handle_as_operand(self);
        let index_address = index_emission.handle_as_operand(self);
        let destination = self.get_next_register();

        let get_list_instruction = Instruction::get_list(
            destination,
            list_address,
            index_address,
            element_type.as_operand_type(),
        );

        Ok(Emission::Instruction(get_list_instruction, element_type))
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

        if matches!(
            left.kind,
            SyntaxKind::BooleanExpression
                | SyntaxKind::ByteExpression
                | SyntaxKind::CharacterExpression
                | SyntaxKind::FloatExpression
                | SyntaxKind::IntegerExpression
                | SyntaxKind::StringExpression
        ) && matches!(
            right.kind,
            SyntaxKind::BooleanExpression
                | SyntaxKind::ByteExpression
                | SyntaxKind::CharacterExpression
                | SyntaxKind::FloatExpression
                | SyntaxKind::IntegerExpression
                | SyntaxKind::StringExpression
        ) {
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
        }

        let left_emission = self.compile_expression(&left)?;
        let left_type = left_emission.r#type().clone();
        let left_address = left_emission.handle_as_operand(self);

        let right_emission = self.compile_expression(&right)?;
        let right_type = right_emission.r#type().clone();
        let right_address = right_emission.handle_as_operand(self);

        let instructions_count_before = self.instructions.len();
        let r#type = if left_type == Type::Character {
            Type::String
        } else {
            left_type.clone()
        };
        let destination = self.get_next_register();
        let operand_type = match (left_type, right_type) {
            (Type::Integer, Type::Integer) => OperandType::INTEGER,
            (Type::Float, Type::Float) => OperandType::FLOAT,
            (Type::Byte, Type::Byte) => OperandType::BYTE,
            (Type::Character, Type::Character) => OperandType::CHARACTER,
            (Type::String, Type::String) => OperandType::STRING,
            (Type::String, Type::Character) => OperandType::STRING_CHARACTER,
            (Type::Character, Type::String) => OperandType::CHARACTER_STRING,
            _ => todo!("Error"),
        };

        let instruction = match node.kind {
            SyntaxKind::AdditionExpression => {
                Instruction::add(destination, left_address, right_address, operand_type)
            }
            SyntaxKind::AdditionAssignmentExpression => {
                self.instructions.truncate(instructions_count_before);

                Instruction::add(
                    left_address.index,
                    left_address,
                    right_address,
                    operand_type,
                )
            }
            SyntaxKind::SubtractionExpression => {
                Instruction::subtract(destination, left_address, right_address, operand_type)
            }
            SyntaxKind::SubtractionAssignmentExpression => {
                self.instructions.truncate(instructions_count_before);

                Instruction::subtract(
                    left_address.index,
                    left_address,
                    right_address,
                    operand_type,
                )
            }
            SyntaxKind::MultiplicationExpression => {
                Instruction::multiply(destination, left_address, right_address, operand_type)
            }
            SyntaxKind::MultiplicationAssignmentExpression => {
                self.instructions.truncate(instructions_count_before);

                Instruction::multiply(
                    left_address.index,
                    left_address,
                    right_address,
                    operand_type,
                )
            }
            SyntaxKind::DivisionExpression => {
                Instruction::divide(destination, left_address, right_address, operand_type)
            }
            SyntaxKind::DivisionAssignmentExpression => {
                self.instructions.truncate(instructions_count_before);

                Instruction::divide(
                    left_address.index,
                    left_address,
                    right_address,
                    operand_type,
                )
            }
            SyntaxKind::ModuloExpression => {
                Instruction::modulo(destination, left_address, right_address, operand_type)
            }
            SyntaxKind::ModuloAssignmentExpression => {
                self.instructions.truncate(instructions_count_before);

                Instruction::modulo(
                    left_address.index,
                    left_address,
                    right_address,
                    operand_type,
                )
            }
            SyntaxKind::ExponentExpression => {
                Instruction::power(destination, left_address, right_address, operand_type)
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

        let destination = self.get_next_register();
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
                let move_instruction =
                    Instruction::r#move(destination, address, operand_type, false);

                (self.handle_operand(move_instruction), operand_type)
            }
            Emission::Local(Local { address, r#type }) => (*address, r#type.as_operand_type()),
            Emission::Function(address, _) => (*address, OperandType::FUNCTION),
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

                self.instructions.extend(instructions.drain(..));

                (destination, operand_type)
            }
            Emission::Constant(constant, type_id) => {
                let r#type = type_id.as_operand_type();
                let address = self.get_constant_address(constant);
                let move_instruction = Instruction::r#move(destination, address, r#type, false);

                (self.handle_operand(move_instruction), r#type)
            }
            Emission::Local(Local { address, r#type }) => (address, r#type.as_operand_type()),
            Emission::Function(address, _) => (address, OperandType::FUNCTION),
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

        let destination = self.get_next_register();
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
        let load_true_instruction = Instruction::r#move(
            destination,
            Address::encoded(true as u16),
            OperandType::BOOLEAN,
            true,
        );
        let load_false_instruction = Instruction::r#move(
            destination,
            Address::encoded(false as u16),
            OperandType::BOOLEAN,
            false,
        );

        Ok(Emission::Instructions(
            smallvec![
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

        let comparator = match node.kind {
            SyntaxKind::AndExpression => true,
            SyntaxKind::OrExpression => false,
            _ => unreachable!("Expected logical expression, found {}", node.kind),
        };

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

        let left_address = match left_emission {
            Emission::Instruction(instruction, _) => {
                self.instructions.push(instruction);

                instruction.destination()
            }
            Emission::Instructions(mut instructions, _) => {
                self.instructions.extend(instructions.drain(..));

                instructions.last().unwrap().destination()
            }
            Emission::Constant(constant, type_id) => {
                let operand_type = type_id.as_operand_type();
                let address = self.get_constant_address(constant);
                let destination = self.get_next_register();
                let move_instruction =
                    Instruction::r#move(destination, address, operand_type, false);

                self.handle_operand(move_instruction)
            }
            Emission::Local(Local { address, .. }) | Emission::Function(address, _) => address,
            Emission::None => {
                return Err(CompileError::ExpectedExpression {
                    node_kind: left.kind,
                    position: Position::new(self.file_id, left.span),
                });
            }
        };
        let right_address = match right_emission {
            Emission::Instruction(instruction, _) => {
                self.instructions.push(instruction);

                instruction.destination()
            }
            Emission::Instructions(mut instructions, _) => {
                self.instructions.extend(instructions.drain(..));

                instructions.last().unwrap().destination()
            }
            Emission::Constant(constant, type_id) => {
                let operand_type = type_id.as_operand_type();
                let address = self.get_constant_address(constant);
                let destination = self.get_next_register();
                let move_instruction =
                    Instruction::r#move(destination, address, operand_type, false);

                self.instructions.push(move_instruction);

                Address::register(destination)
            }
            Emission::Local(Local { address, .. }) | Emission::Function(address, _) => address,
            Emission::None => {
                return Err(CompileError::ExpectedExpression {
                    node_kind: right.kind,
                    position: Position::new(self.file_id, right.span),
                });
            }
        };

        let mut instructions = SmallVec::new();

        let load_destination = self.get_next_register();
        let left_move_instruction =
            Instruction::r#move(load_destination, left_address, OperandType::BOOLEAN, false);
        let test_instruction = Instruction::test(Address::register(load_destination), comparator);
        let jump_instruction = Instruction::jump(1, true);
        let right_move_instruction =
            Instruction::r#move(load_destination, right_address, OperandType::BOOLEAN, false);

        instructions.push(left_move_instruction);
        instructions.push(test_instruction);
        instructions.push(jump_instruction);
        instructions.push(right_move_instruction);

        let emission = Emission::Instructions(instructions, Type::Boolean);

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
        let destination = self.get_next_register();
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

        let start_scope_id = self.current_scope_id;
        let new_scope_id = self.context.resolver.add_scope(Scope {
            kind: ScopeKind::Block,
            parent: start_scope_id,
            imports: SmallVec::new(),
            modules: SmallVec::new(),
        });
        self.current_scope_id = new_scope_id;

        let mut block_instructions = SmallVec::<[Instruction; 8]>::new();
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
            let child_node =
                *self
                    .syntax_tree()?
                    .get_node(child_id)
                    .ok_or(CompileError::MissingSyntaxNode {
                        syntax_id: child_id,
                    })?;
            current_child_index += 1;

            let statment_emission = self.compile_statement(&child_node)?;

            match statment_emission {
                Emission::Instruction(instruction, _) => {
                    block_instructions.push(instruction);
                }
                Emission::Instructions(mut instructions, _) => {
                    block_instructions.extend(instructions.drain(..));
                }
                Emission::Constant(constant, type_id) => {
                    let operand_type = type_id.as_operand_type();
                    let address = self.get_constant_address(constant);
                    let destination = self.get_next_register();
                    let move_instruction =
                        Instruction::r#move(destination, address, operand_type, false);

                    block_instructions.push(move_instruction);
                }
                Emission::Local(Local { address, r#type }) => {
                    let move_instruction = Instruction::r#move(
                        self.get_next_register(),
                        address,
                        r#type.as_operand_type(),
                        false,
                    );

                    block_instructions.push(move_instruction);
                }
                Emission::Function(address, _) => {
                    let move_instruction = Instruction::r#move(
                        self.get_next_register(),
                        address,
                        OperandType::FUNCTION,
                        false,
                    );

                    block_instructions.push(move_instruction);
                }
                Emission::None => {}
            }
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
        let last_child_node = *self.syntax_tree()?.get_node(last_child_id).ok_or(
            CompileError::MissingSyntaxNode {
                syntax_id: last_child_id,
            },
        )?;

        self.current_scope_id = start_scope_id;

        let emission = if last_child_node.kind.is_expression() {
            self.compile_expression(&last_child_node)?
        } else {
            self.compile_statement(&last_child_node)?;

            Emission::None
        };

        let block_type = match emission {
            Emission::Instruction(instruction, r#type) => {
                block_instructions.push(instruction);

                r#type
            }
            Emission::Instructions(mut instructions, r#type) => {
                block_instructions.extend(instructions.drain(..));

                r#type
            }
            Emission::Constant(constant, r#type) => {
                let operand_type = r#type.as_operand_type();
                let address = self.get_constant_address(constant);
                let destination = self.get_next_register();
                let move_instruction =
                    Instruction::r#move(destination, address, operand_type, false);

                block_instructions.push(move_instruction);

                r#type
            }
            Emission::Local(Local { address, r#type }) => {
                let move_instruction = Instruction::r#move(
                    self.get_next_register(),
                    address,
                    r#type.as_operand_type(),
                    false,
                );

                block_instructions.push(move_instruction);

                r#type
            }
            Emission::Function(address, r#type) => {
                let move_instruction = Instruction::r#move(
                    self.get_next_register(),
                    address,
                    OperandType::FUNCTION,
                    false,
                );

                block_instructions.push(move_instruction);

                r#type
            }
            Emission::None => Type::None,
        };

        Ok(Emission::Instructions(block_instructions, block_type))
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
        let variable_name = unsafe { str::from_utf8_unchecked(variable_name_bytes) };

        let (declaration_id, declaration) = self
            .context
            .resolver
            .find_declaration_in_scope(variable_name, self.current_scope_id)
            .ok_or(CompileError::UndeclaredVariable {
                name: unsafe { String::from_utf8_unchecked(variable_name_bytes.to_vec()) },
                position: Position::new(self.file_id, node.span),
            })?;

        drop(files);

        let local = if let Some(local) = self.locals.get(&declaration_id) {
            local.clone()
        } else {
            let r#type = self
                .context
                .resolver
                .resolve_type(declaration.type_id)
                .ok_or(CompileError::MissingType {
                    type_id: declaration.type_id,
                })?
                .clone();
            let address = if matches!(declaration.kind, DeclarationKind::Function { .. }) {
                if let Some(index) = self
                    .context
                    .prototypes
                    .iter()
                    .position(|chunk| chunk.name_position == Some(declaration.position))
                {
                    Address::constant(index as u16)
                } else {
                    Address::register(self.get_next_register())
                }
            } else {
                Address::register(self.get_next_register())
            };
            let local = Local { r#type, address };

            self.locals.insert(declaration_id, local.clone());

            local
        };

        if matches!(declaration.kind, DeclarationKind::Function { .. }) {
            let function_type = local.r#type.clone();

            Ok(Emission::Function(local.address, function_type))
        } else {
            Ok(Emission::Local(local))
        }
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
                let destination = self.get_next_register();
                let move_instruction =
                    Instruction::r#move(destination, address, operand_type, false);

                self.handle_operand(move_instruction);
            }
            Emission::Local(Local { address, .. }) => {
                let test_instruction = Instruction::test(address, true);

                self.instructions.push(test_instruction);
            }
            Emission::Function(address, _) => {
                let test_instruction = Instruction::test(address, true);

                self.instructions.push(test_instruction);
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
        declaration_info: Option<(DeclarationId, Declaration)>,
        bound_type: Option<FunctionType>,
    ) -> Result<Emission, CompileError> {
        info!("Compiling function expression");

        let r#type = if let Some(function_type) = bound_type {
            function_type
        } else {
            let function_signature_id = SyntaxId(node.children.0);
            let function_signature_node = *self
                .syntax_tree()?
                .get_node(function_signature_id)
                .ok_or(CompileError::MissingSyntaxNode {
                    syntax_id: function_signature_id,
                })?;

            debug_assert_eq!(function_signature_node.kind, SyntaxKind::FunctionSignature);

            let value_parameter_list_id = SyntaxId(function_signature_node.children.0);
            let value_parameter_list_node = *self
                .syntax_tree()?
                .get_node(value_parameter_list_id)
                .ok_or(CompileError::MissingSyntaxNode {
                    syntax_id: value_parameter_list_id,
                })?;

            debug_assert_eq!(
                value_parameter_list_node.kind,
                SyntaxKind::FunctionValueParameters
            );

            let function_scope = self.context.resolver.add_scope(Scope {
                kind: ScopeKind::Function,
                parent: self.current_scope_id,
                imports: SmallVec::new(),
                modules: SmallVec::new(),
            });
            let value_parameter_node_ids = self
                .syntax_tree()?
                .get_children(
                    value_parameter_list_node.children.0,
                    value_parameter_list_node.children.1,
                )
                .ok_or(CompileError::MissingChildren {
                    parent_kind: value_parameter_list_node.kind,
                    start_index: value_parameter_list_node.children.0,
                    count: value_parameter_list_node.children.1,
                })?;
            let value_parameter_nodes = value_parameter_node_ids
                .iter()
                .map(|&id| {
                    self.syntax_tree()?
                        .get_node(id)
                        .ok_or(CompileError::MissingSyntaxNode { syntax_id: id })
                        .copied()
                })
                .collect::<Result<SmallVec<[SyntaxNode; 4]>, CompileError>>()?;

            let files = &self.context.source.read_files();
            let file =
                files
                    .get(self.file_id.0 as usize)
                    .ok_or(CompileError::MissingSourceFile {
                        file_id: self.file_id,
                    })?;

            let mut value_parameters = Vec::new();
            let mut current_parameter_name = "";

            for (index, node) in value_parameter_nodes.iter().enumerate() {
                let is_name = index % 2 == 0;

                if is_name {
                    current_parameter_name = unsafe {
                        str::from_utf8_unchecked(
                            &file.source_code.as_ref()[node.span.0 as usize..node.span.1 as usize],
                        )
                    };
                } else {
                    let r#type = match node.kind {
                        SyntaxKind::BooleanType => Type::Boolean,
                        SyntaxKind::ByteType => Type::Byte,
                        SyntaxKind::CharacterType => Type::Character,
                        SyntaxKind::FloatType => Type::Float,
                        SyntaxKind::IntegerType => Type::Integer,
                        SyntaxKind::StringType => Type::String,
                        _ => {
                            todo!()
                        }
                    };
                    let type_id = self.context.resolver.add_type(&r#type);
                    let parameter_declaration = Declaration {
                        kind: DeclarationKind::Local { shadowed: None },
                        scope_id: function_scope,
                        type_id,
                        position: Position::new(self.file_id, node.span),
                        is_public: false,
                    };

                    self.context
                        .resolver
                        .add_declaration(current_parameter_name, parameter_declaration);
                    value_parameters.push(r#type);
                }
            }

            let function_return_type_id = SyntaxId(function_signature_node.children.1);
            let return_type = {
                if function_return_type_id == SyntaxId::NONE {
                    Type::None
                } else {
                    let function_return_type_node = *self
                        .syntax_tree()?
                        .get_node(function_return_type_id)
                        .ok_or(CompileError::MissingSyntaxNode {
                            syntax_id: function_return_type_id,
                        })?;

                    match function_return_type_node.kind {
                        SyntaxKind::BooleanType => Type::Boolean,
                        SyntaxKind::ByteType => Type::Byte,
                        SyntaxKind::CharacterType => Type::Character,
                        SyntaxKind::FloatType => Type::Float,
                        SyntaxKind::IntegerType => Type::Integer,
                        SyntaxKind::StringType => Type::String,
                        _ => {
                            todo!()
                        }
                    }
                }
            };

            FunctionType {
                type_parameters: Vec::new(),
                value_parameters,
                return_type,
            }
        };

        let block_id = SyntaxId(node.children.1);
        let body_node =
            *self
                .syntax_tree()?
                .get_node(block_id)
                .ok_or(CompileError::MissingSyntaxNode {
                    syntax_id: block_id,
                })?;
        let (declaration_id, function_scope_id) =
            if let Some((declaration_id, declaration)) = declaration_info {
                let scope_id = match declaration.kind {
                    DeclarationKind::Function { inner_scope_id, .. } => inner_scope_id,
                    _ => declaration.scope_id,
                };

                (Some(declaration_id), scope_id)
            } else {
                let function_scope = self.context.resolver.add_scope(Scope {
                    kind: ScopeKind::Function,
                    parent: self.current_scope_id,
                    imports: SmallVec::new(),
                    modules: SmallVec::new(),
                });

                (None, function_scope)
            };

        let mut function_compiler = ChunkCompiler::new(
            declaration_id,
            self.file_id,
            r#type.clone(),
            self.context,
            function_scope_id,
        );

        function_compiler.compile_implicit_return(&body_node)?;

        let function_chunk = function_compiler.finish()?;
        let prototype_index = self.context.prototypes.len() as u16;
        let r#type = Type::Function(Box::new(r#type));

        self.context.prototypes.push(function_chunk);

        Ok(Emission::Function(
            Address::constant(prototype_index),
            r#type,
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
                let child_node = *compiler.syntax_tree()?.get_node(child_id).ok_or(
                    CompileError::MissingSyntaxNode {
                        syntax_id: child_id,
                    },
                )?;
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

        if function_node.kind == SyntaxKind::PathExpression {
            let destination = self.get_next_register();
            let native_function = {
                let files = self.context.source.read_files();
                let source_file =
                    files
                        .get(self.file_id.0 as usize)
                        .ok_or(CompileError::MissingSourceFile {
                            file_id: self.file_id,
                        })?;
                let name_bytes = &source_file.source_code.as_ref()
                    [function_node.span.0 as usize..function_node.span.1 as usize];

                if name_bytes == b"write_line" {
                    Some(NativeFunction::WRITE_LINE)
                } else if name_bytes == b"read_line" {
                    Some(NativeFunction::READ_LINE)
                } else {
                    None
                }
            };

            if let Some(native_function) = native_function {
                let arguments_start_index = self.call_arguments.len() as u16;

                handle_call_arguments(self, &arguments_node)?;

                let call_native_instruction =
                    Instruction::call_native(destination, native_function, arguments_start_index);
                let function_type = native_function.r#type();

                return Ok(Emission::Instruction(
                    call_native_instruction,
                    function_type.return_type,
                ));
            }
        }

        let expression_emission = self.compile_expression(&function_node)?;

        let Emission::Function(address, r#type) = expression_emission else {
            return Err(CompileError::ExpectedFunction {
                node_kind: function_node.kind,
                position: Position::new(self.file_id, function_node.span),
            });
        };

        let arguments_start_index = self.call_arguments.len() as u16;

        handle_call_arguments(self, &arguments_node)?;

        let destination_index = self.get_next_register();
        let r#type = r#type
            .into_function_type()
            .ok_or(CompileError::ExpectedFunction {
                node_kind: function_node.kind,
                position: Position::new(self.file_id, function_node.span),
            })?
            .return_type
            .clone();
        let argument_count = self.call_arguments.len() as u16 - arguments_start_index;
        let call_instruction = Instruction::call(
            destination_index,
            address.index,
            arguments_start_index,
            argument_count,
            r#type.as_operand_type(),
        );

        Ok(Emission::Instruction(call_instruction, r#type))
    }

    fn compile_as_expression(&mut self, node: &SyntaxNode) -> Result<Emission, CompileError> {
        info!("Compiling 'as' expression");

        let value_node_id = SyntaxId(node.children.0);
        let type_node_id = SyntaxId(node.children.1);

        let value_node =
            *self
                .syntax_tree()?
                .get_node(value_node_id)
                .ok_or(CompileError::MissingChild {
                    parent_kind: node.kind,
                    child_index: node.children.0,
                })?;
        let type_node =
            *self
                .syntax_tree()?
                .get_node(type_node_id)
                .ok_or(CompileError::MissingChild {
                    parent_kind: node.kind,
                    child_index: node.children.1,
                })?;

        let value_emission = self.compile_expression(&value_node)?;
        let value_type = value_emission.r#type().as_operand_type();
        let value_address = value_emission.handle_as_operand(self);
        let destination = self.get_next_register();
        let to_type_instruction = match type_node.kind {
            SyntaxKind::StringType => {
                Instruction::to_string(destination, value_address, value_type)
            }
            _ => {
                todo!()
            }
        };

        Ok(Emission::Instruction(to_type_instruction, Type::String))
    }

    fn compile_if_expression(&mut self, node: &SyntaxNode) -> Result<Emission, CompileError> {
        info!("Compiling if expression");

        let children_ids = self
            .syntax_tree()?
            .get_children(node.children.0, node.children.1)
            .ok_or(CompileError::MissingChildren {
                parent_kind: node.kind,
                start_index: node.children.0,
                count: node.children.1,
            })?
            .iter()
            .cloned()
            .collect::<SmallVec<[SyntaxId; 3]>>();

        let condition_id = children_ids[0];
        let then_block_id = children_ids[1];
        let condition_node =
            *self
                .syntax_tree()?
                .get_node(condition_id)
                .ok_or(CompileError::MissingChild {
                    parent_kind: node.kind,
                    child_index: node.children.0,
                })?;
        let then_block_node =
            *self
                .syntax_tree()?
                .get_node(then_block_id)
                .ok_or(CompileError::MissingChild {
                    parent_kind: node.kind,
                    child_index: node.children.1,
                })?;
        let condition_emission = self.compile_expression(&condition_node)?;
        let mut instructions = SmallVec::new();
        let (condition_address, conditon_type, needs_test) = match condition_emission {
            Emission::Instruction(instruction, r#type) => {
                instructions.push(instruction);

                (instruction.destination(), r#type, true)
            }
            Emission::Instructions(then_instructions, r#type) => {
                let comparison_or_test_instruction = then_instructions[0];

                instructions.push(comparison_or_test_instruction);

                (comparison_or_test_instruction.destination(), r#type, false)
            }
            Emission::Constant(constant, r#type) => {
                let operand_type = r#type.as_operand_type();
                let address = self.get_constant_address(constant);
                let destination = self.get_next_register();
                let move_instruction =
                    Instruction::r#move(destination, address, operand_type, false);

                instructions.push(move_instruction);

                (Address::register(destination), r#type, true)
            }
            Emission::Local(Local { address, r#type }) => (address, r#type, true),
            _ => {
                return Err(CompileError::ExpectedBooleanExpression {
                    node_kind: condition_node.kind,
                    position: Position::new(self.file_id, condition_node.span),
                });
            }
        };

        if conditon_type != Type::Boolean {
            return Err(CompileError::ExpectedBooleanExpression {
                node_kind: condition_node.kind,
                position: Position::new(self.file_id, condition_node.span),
            });
        }

        if needs_test {
            let test_instruction = Instruction::test(condition_address, true);

            instructions.push(test_instruction);
        }

        let jump_to_else_index = instructions.len();

        instructions.push(Instruction::no_op());

        let then_emission = self.compile_expression(&then_block_node)?;
        let emission_type = match then_emission {
            Emission::Instruction(instruction, r#type) => {
                instructions.push(instruction);

                r#type
            }
            Emission::Instructions(mut then_instructions, r#type) => {
                instructions.extend(then_instructions.drain(..));

                r#type
            }
            Emission::Constant(constant, r#type) => {
                let operand_type = r#type.as_operand_type();
                let address = self.get_constant_address(constant);
                let destination = self.get_next_register();
                let move_instruction =
                    Instruction::r#move(destination, address, operand_type, false);

                instructions.push(move_instruction);

                r#type
            }
            Emission::Local(Local { address, r#type }) => {
                let move_instruction = Instruction::r#move(
                    self.get_next_register(),
                    address,
                    r#type.as_operand_type(),
                    false,
                );

                instructions.push(move_instruction);

                r#type
            }
            Emission::Function(address, r#type) => {
                let move_instruction = Instruction::r#move(
                    self.get_next_register(),
                    address,
                    r#type.as_operand_type(),
                    false,
                );

                instructions.push(move_instruction);

                r#type
            }
            Emission::None => {
                return Err(CompileError::ExpectedExpression {
                    node_kind: then_block_node.kind,
                    position: Position::new(self.file_id, then_block_node.span),
                });
            }
        };

        if children_ids.len() == 3 {
            let jump_to_end_index = instructions.len();

            instructions.push(Instruction::no_op());

            let distance_to_else = (instructions.len() - jump_to_else_index - 1) as u16;
            let jump_to_else_instruction = Instruction::jump(distance_to_else, true);
            instructions[jump_to_else_index] = jump_to_else_instruction;
            let else_block_id = children_ids[2];
            let else_block_node =
                *self
                    .syntax_tree()?
                    .get_node(else_block_id)
                    .ok_or(CompileError::MissingChild {
                        parent_kind: node.kind,
                        child_index: node.children.0 + 2,
                    })?;
            let else_emission = self.compile_else_expression(&else_block_node)?;

            match else_emission {
                Emission::Instruction(instruction, r#type) => {
                    instructions.push(instruction);

                    if r#type != emission_type {
                        return Err(CompileError::MismatchedIfElseTypes {
                            then_type: emission_type,
                            else_type: r#type,
                            position: Position::new(self.file_id, else_block_node.span),
                        });
                    }
                }
                Emission::Instructions(mut else_instructions, r#type) => {
                    instructions.extend(else_instructions.drain(..));

                    if r#type != emission_type {
                        return Err(CompileError::MismatchedIfElseTypes {
                            then_type: emission_type,
                            else_type: r#type,
                            position: Position::new(self.file_id, else_block_node.span),
                        });
                    }
                }
                Emission::Constant(constant, r#type) => {
                    let operand_type = r#type.as_operand_type();
                    let address = self.get_constant_address(constant);
                    let destination = self.get_next_register();
                    let move_instruction =
                        Instruction::r#move(destination, address, operand_type, false);

                    instructions.push(move_instruction);

                    if r#type != emission_type {
                        return Err(CompileError::MismatchedIfElseTypes {
                            then_type: emission_type,
                            else_type: r#type,
                            position: Position::new(self.file_id, else_block_node.span),
                        });
                    }
                }
                Emission::Local(Local { address, r#type }) => {
                    let move_instruction = Instruction::r#move(
                        self.get_next_register(),
                        address,
                        r#type.as_operand_type(),
                        false,
                    );

                    instructions.push(move_instruction);

                    if r#type != emission_type {
                        return Err(CompileError::MismatchedIfElseTypes {
                            then_type: emission_type,
                            else_type: r#type,
                            position: Position::new(self.file_id, else_block_node.span),
                        });
                    }
                }
                _ => {
                    return Err(CompileError::ExpectedExpression {
                        node_kind: else_block_node.kind,
                        position: Position::new(self.file_id, else_block_node.span),
                    });
                }
            }

            let distance_to_end = (instructions.len() - jump_to_end_index - 1) as u16;
            let jump_to_end_instruction = Instruction::jump(distance_to_end, true);
            instructions[jump_to_end_index] = jump_to_end_instruction;
        } else {
            let distance_past_then = (instructions.len() - jump_to_else_index - 1) as u16;
            let jump_forward_instruction = Instruction::jump(distance_past_then, true);
            instructions[jump_to_else_index] = jump_forward_instruction;
        }

        Ok(Emission::Instructions(instructions, emission_type))
    }

    fn compile_else_expression(&mut self, node: &SyntaxNode) -> Result<Emission, CompileError> {
        info!("Compiling else expression");

        let child_id = SyntaxId(node.children.0);
        let child_node =
            *self
                .syntax_tree()?
                .get_node(child_id)
                .ok_or(CompileError::MissingChild {
                    parent_kind: node.kind,
                    child_index: node.children.0,
                })?;

        if child_node.kind == SyntaxKind::IfExpression {
            self.compile_if_expression(&child_node)
        } else {
            self.compile_block_expression(&child_node)
        }
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
                        && last_instruction.operation() == Operation::MOVE
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
                    let address = self.get_constant_address(constant);

                    (address, r#type)
                }
                Emission::Function(address, function_type) => (address, function_type),
                Emission::Local(Local { address, r#type }) => {
                    let mut return_address = address;

                    if let Some(last_instruction) = self.instructions.last()
                        && last_instruction.operation() == Operation::MOVE
                        && last_instruction.destination() == return_address
                    {
                        return_address = last_instruction.b_address();

                        self.instructions.pop();
                    }

                    (return_address, r#type)
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
enum Emission {
    Instruction(Instruction, Type),
    Instructions(SmallVec<[Instruction; 8]>, Type),
    Local(Local),
    Constant(Constant, Type),
    Function(Address, Type),
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
            Emission::Constant(constant, type_node) => {
                let address = compiler.get_constant_address(constant);
                let destination = compiler.get_next_register();
                let move_instruction =
                    Instruction::r#move(destination, address, type_node.as_operand_type(), false);

                compiler.handle_operand(move_instruction)
            }
            Emission::Local(Local { address, .. }) => address,
            Emission::Function(address, _) => address,
            Emission::None => Address::default(),
        }
    }

    fn r#type(&self) -> &Type {
        match self {
            Emission::Instruction(_, r#type) => r#type,
            Emission::Instructions(_, r#type) => r#type,
            Emission::Constant(_, r#type) => r#type,
            Emission::Local(Local { r#type, .. }) => r#type,
            Emission::Function(_, r#type) => r#type,
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
}

#[derive(Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
struct Local {
    address: Address,
    r#type: Type,
}
