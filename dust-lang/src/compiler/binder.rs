use smallvec::SmallVec;
use tracing::{Level, info, span};

use crate::{
    compiler::CompileError,
    resolver::{
        Declaration, DeclarationId, DeclarationKind, FunctionTypeNode, ModuleKind, Resolver, Scope,
        ScopeId, ScopeKind, TypeId, TypeNode,
    },
    source::{Position, Source, SourceFileId, Span},
    syntax_tree::{SyntaxId, SyntaxKind, SyntaxNode, SyntaxTree},
};

pub struct Binder<'a> {
    file_id: SourceFileId,

    source: Source,

    resolver: &'a mut Resolver,

    syntax_tree: &'a SyntaxTree,

    current_scope_id: ScopeId,
}

impl<'a> Binder<'a> {
    pub fn new(
        file_id: SourceFileId,
        source: Source,
        resolver: &'a mut Resolver,
        syntax_tree: &'a SyntaxTree,
        current_scope_id: ScopeId,
    ) -> Self {
        Self {
            file_id,
            source,
            resolver,
            syntax_tree,
            current_scope_id,
        }
    }

    pub fn bind_main(mut self) -> Result<(), CompileError> {
        let span = span!(Level::INFO, "bind_main");
        let _enter = span.enter();

        let root_node =
            *self
                .syntax_tree
                .get_node(SyntaxId(0))
                .ok_or(CompileError::MissingSyntaxNode {
                    syntax_id: SyntaxId(0),
                })?;

        self.bind_main_function_item(&root_node)?;

        Ok(())
    }

    pub fn bind_file_module(mut self, module_name: &str) -> Result<(), CompileError> {
        let span = span!(Level::INFO, "bind_module");
        let _enter = span.enter();

        let declaration = Declaration {
            kind: DeclarationKind::Module {
                kind: ModuleKind::File,
                inner_scope_id: self.current_scope_id,
            },
            scope_id: ScopeId::MAIN,
            type_id: TypeId::NONE,
            position: Position::new(self.file_id, Span(0, 0)),
            is_public: true,
        };
        let root_node = *self
            .syntax_tree
            .top_node()
            .ok_or(CompileError::MissingSyntaxNode {
                syntax_id: SyntaxId(0),
            })?;
        let child_ids = self
            .syntax_tree
            .get_children(root_node.children.0, root_node.children.1)
            .ok_or(CompileError::MissingChildren {
                parent_kind: root_node.kind,
                start_index: root_node.children.0,
                count: root_node.children.1,
            })?;

        let mut current_child_index = 0;

        while current_child_index < child_ids.len() {
            let child_id = child_ids[current_child_index];
            let child_node =
                *self
                    .syntax_tree
                    .get_node(child_id)
                    .ok_or(CompileError::MissingSyntaxNode {
                        syntax_id: child_id,
                    })?;
            current_child_index += 1;

            if child_node.kind.is_item() {
                self.bind_item(child_id, &child_node)?;
            }
        }

        self.resolver.add_declaration(module_name, declaration);
        self.bind_module_item(&root_node)?;

        Ok(())
    }

    fn bind_item(&mut self, node_id: SyntaxId, node: &SyntaxNode) -> Result<(), CompileError> {
        match node.kind {
            SyntaxKind::MainFunctionItem => self.bind_main_function_item(node)?,
            SyntaxKind::FunctionItem => self.bind_function_item(node_id, node)?,
            SyntaxKind::PublicFunctionItem => self.bind_function_item(node_id, node)?,
            SyntaxKind::ModuleItem => self.bind_module_item(node)?,
            SyntaxKind::PublicModuleItem => self.bind_module_item(node)?,
            SyntaxKind::UseItem => self.bind_use_item(node)?,
            _ => todo!(),
        }

        Ok(())
    }

    fn bind_main_function_item(&mut self, node: &SyntaxNode) -> Result<(), CompileError> {
        info!("Binding main function");

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
            let child_node =
                self.syntax_tree
                    .get_node(child_id)
                    .ok_or(CompileError::MissingSyntaxNode {
                        syntax_id: child_id,
                    })?;
            current_child_index += 1;

            if child_node.kind.is_item() {
                self.bind_item(child_id, child_node)?;
            } else if child_node.kind.is_statement() {
                self.bind_statement(child_id, child_node)?;
            } else if child_node.kind.is_expression() {
                self.bind_expression(child_id, child_node)?;
            }
        }

        Ok(())
    }

    fn bind_module_item(&mut self, node: &SyntaxNode) -> Result<(), CompileError> {
        info!("Binding module");

        let (start_children, child_count) = (node.children.0, node.children.1);
        let children = self
            .syntax_tree
            .get_children(start_children, child_count)
            .ok_or(CompileError::MissingChildren {
                parent_kind: node.kind,
                start_index: node.children.0,
                count: node.children.1,
            })?;

        for &child_id in children.iter() {
            let child_node =
                *self
                    .syntax_tree
                    .get_node(child_id)
                    .ok_or(CompileError::MissingSyntaxNode {
                        syntax_id: child_id,
                    })?;

            if child_node.kind.is_item() {
                self.bind_item(child_id, &child_node)?;
            }
        }

        Ok(())
    }

    pub fn bind_function_item(
        &mut self,
        node_id: SyntaxId,
        node: &SyntaxNode,
    ) -> Result<(), CompileError> {
        info!("Binding function item");

        debug_assert!(matches!(
            node.kind,
            SyntaxKind::FunctionItem | SyntaxKind::PublicFunctionItem
        ));

        let function_expression_id = SyntaxId(node.children.1);
        let function_expression_node = *self.syntax_tree.get_node(function_expression_id).ok_or(
            CompileError::MissingSyntaxNode {
                syntax_id: function_expression_id,
            },
        )?;

        debug_assert_eq!(
            function_expression_node.kind,
            SyntaxKind::FunctionExpression
        );

        let function_signature_id = SyntaxId(function_expression_node.children.0);
        let function_signature_node = *self.syntax_tree.get_node(function_signature_id).ok_or(
            CompileError::MissingSyntaxNode {
                syntax_id: function_signature_id,
            },
        )?;

        debug_assert_eq!(function_signature_node.kind, SyntaxKind::FunctionSignature);

        let value_parameter_list_id = SyntaxId(function_signature_node.children.0);
        let value_parameter_list_node = *self.syntax_tree.get_node(value_parameter_list_id).ok_or(
            CompileError::MissingSyntaxNode {
                syntax_id: value_parameter_list_id,
            },
        )?;

        debug_assert_eq!(
            value_parameter_list_node.kind,
            SyntaxKind::ValueParametersDefinition
        );

        let value_parameter_node_ids = self
            .syntax_tree
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
                self.syntax_tree
                    .get_node(id)
                    .ok_or(CompileError::MissingSyntaxNode { syntax_id: id })
            })
            .collect::<Result<SmallVec<[&SyntaxNode; 4]>, CompileError>>()?;

        let function_scope = self.resolver.add_scope(Scope {
            kind: ScopeKind::Function,
            parent: self.current_scope_id,
            imports: SmallVec::new(),
            modules: SmallVec::new(),
        });
        let mut value_parameters = Vec::new();
        let mut parameter_ids = SmallVec::<[DeclarationId; 4]>::new();

        {
            let files = self.source.read_files();
            let file =
                files
                    .get(self.file_id.0 as usize)
                    .ok_or(CompileError::MissingSourceFile {
                        file_id: self.file_id,
                    })?;

            for node in value_parameter_nodes {
                let parameter_name_node_id = node.children.0;
                let parameter_name_node = *self
                    .syntax_tree
                    .get_node(SyntaxId(parameter_name_node_id))
                    .ok_or(CompileError::MissingSyntaxNode {
                        syntax_id: SyntaxId(parameter_name_node_id),
                    })?;

                let parameter_name = unsafe {
                    str::from_utf8_unchecked(
                        &file.source_code.as_ref()[parameter_name_node.span.0 as usize
                            ..parameter_name_node.span.1 as usize],
                    )
                };

                let parameter_type_node_id = node.children.1;
                let parameter_type_node = *self
                    .syntax_tree
                    .get_node(SyntaxId(parameter_type_node_id))
                    .ok_or(CompileError::MissingSyntaxNode {
                        syntax_id: SyntaxId(parameter_type_node_id),
                    })?;

                let type_id =
                    Self::get_type_id(&parameter_type_node, self.syntax_tree, self.resolver)?;
                let parameter_declaration = Declaration {
                    kind: DeclarationKind::Local { shadowed: None },
                    scope_id: function_scope,
                    type_id,
                    position: Position::new(self.file_id, parameter_name_node.span),
                    is_public: false,
                };

                value_parameters.push(type_id);

                let parameter_id = self
                    .resolver
                    .add_declaration(parameter_name, parameter_declaration);

                parameter_ids.push(parameter_id);
            }
        }

        let function_return_type_node_id = SyntaxId(function_signature_node.children.1);
        let return_type_id = if function_return_type_node_id == SyntaxId::NONE {
            TypeId::NONE
        } else {
            let function_return_type_node = *self
                .syntax_tree
                .get_node(function_return_type_node_id)
                .ok_or(CompileError::MissingSyntaxNode {
                    syntax_id: function_return_type_node_id,
                })?;

            Self::get_type_id(&function_return_type_node, self.syntax_tree, self.resolver)?
        };

        let function_type_node = FunctionTypeNode {
            type_parameters: (0, 0),
            value_parameters: self.resolver.add_type_members(&value_parameters),
            return_type: return_type_id,
        };
        let function_type_id = self
            .resolver
            .add_type_node(TypeNode::Function(function_type_node));

        let path_id = SyntaxId(node.children.0);
        let path_node = *self
            .syntax_tree
            .get_node(path_id)
            .ok_or(CompileError::MissingSyntaxNode { syntax_id: path_id })?;

        debug_assert_eq!(path_node.kind, SyntaxKind::Path);

        let path_str = {
            let files = self.source.read_files();
            let path_bytes = &files
                .get(self.file_id.0 as usize)
                .ok_or(CompileError::MissingSourceFile {
                    file_id: self.file_id,
                })?
                .source_code
                .as_ref()[path_node.span.0 as usize..path_node.span.1 as usize];
            unsafe { str::from_utf8_unchecked(path_bytes) }.to_string()
        };

        let parameter_children = self.resolver.add_parameters(&parameter_ids);
        let declaration = Declaration {
            kind: DeclarationKind::Function {
                inner_scope_id: function_scope,
                syntax_id: node_id,
                file_id: self.file_id,
                parameters: parameter_children,
                prototype_index: None,
            },
            scope_id: self.current_scope_id,
            type_id: function_type_id,
            position: Position::new(self.file_id, path_node.span),
            is_public: true,
        };

        self.resolver.add_declaration(&path_str, declaration);

        // Bind the function body
        let function_body_id = SyntaxId(function_expression_node.children.1);
        let function_body_node = *self.syntax_tree.get_node(function_body_id).ok_or(
            CompileError::MissingSyntaxNode {
                syntax_id: function_body_id,
            },
        )?;

        let parent_scope_id = self.current_scope_id;
        self.current_scope_id = function_scope;

        self.bind_function(node_id, &function_body_node)?;

        self.current_scope_id = parent_scope_id;

        Ok(())
    }

    fn bind_use_item(&mut self, node: &SyntaxNode) -> Result<(), CompileError> {
        info!("Binding use item");

        let path_id = SyntaxId(node.children.0);
        let path_node = *self
            .syntax_tree
            .get_node(path_id)
            .ok_or(CompileError::MissingSyntaxNode { syntax_id: path_id })?;

        let path_segment_ids = self
            .syntax_tree
            .get_children(path_node.children.0, path_node.children.1)
            .ok_or(CompileError::MissingChildren {
                parent_kind: path_node.kind,
                start_index: path_node.children.0,
                count: path_node.children.1,
            })?;
        let path_segment_nodes = path_segment_ids
            .iter()
            .map(|&id| {
                self.syntax_tree
                    .get_node(id)
                    .ok_or(CompileError::MissingSyntaxNode { syntax_id: id })
            })
            .collect::<Result<SmallVec<[&SyntaxNode; 4]>, CompileError>>()?;

        let files = &self.source.read_files();

        let mut current_segment_index = 0;
        let mut current_scope_id = self.current_scope_id;
        let mut current_declaration_id;

        loop {
            let segment_node = path_segment_nodes[current_segment_index];

            let segment_name_bytes = &files
                .get(self.file_id.0 as usize)
                .ok_or(CompileError::MissingSourceFile {
                    file_id: self.file_id,
                })?
                .source_code
                .as_ref()[segment_node.span.0 as usize..segment_node.span.1 as usize];
            let segment_name = unsafe { str::from_utf8_unchecked(segment_name_bytes) };

            let (declaration_id, declaration) = self
                .resolver
                .find_declaration_in_scope(segment_name, current_scope_id)
                .ok_or(CompileError::UndeclaredVariable {
                    name: segment_name.to_string(),
                    position: Position::new(self.file_id, segment_node.span),
                })?;

            current_declaration_id = declaration_id;
            current_scope_id =
                if let DeclarationKind::Module { inner_scope_id, .. } = &declaration.kind {
                    *inner_scope_id
                } else {
                    declaration.scope_id
                };
            current_segment_index += 1;

            if current_segment_index == path_segment_nodes.len() {
                drop(path_segment_nodes);

                break;
            }
        }

        self.resolver
            .get_scope_mut(self.current_scope_id)
            .ok_or(CompileError::MissingScope {
                scope_id: self.current_scope_id,
            })?
            .imports
            .push(current_declaration_id);

        Ok(())
    }

    fn bind_function(&mut self, node_id: SyntaxId, node: &SyntaxNode) -> Result<(), CompileError> {
        info!("Binding function");

        let (start_children, child_count) = match node.kind {
            SyntaxKind::BlockExpression => (node.children.0 as usize, node.children.1 as usize),
            SyntaxKind::ExpressionStatement => {
                let expression_id = SyntaxId(node.children.0);
                let expression_node = *self.syntax_tree.get_node(expression_id).ok_or(
                    CompileError::MissingSyntaxNode {
                        syntax_id: expression_id,
                    },
                )?;

                if expression_node.kind == SyntaxKind::BlockExpression {
                    (
                        expression_node.children.0 as usize,
                        expression_node.children.1 as usize,
                    )
                } else {
                    self.bind_expression(node_id, &expression_node)?;

                    return Ok(());
                }
            }
            _ => {
                if node.kind.is_expression() {
                    self.bind_expression(node_id, node)?;
                }

                return Ok(());
            }
        };

        let end_children = start_children + child_count;

        if child_count == 0 {
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
            let child_node =
                *self
                    .syntax_tree
                    .get_node(child_id)
                    .ok_or(CompileError::MissingSyntaxNode {
                        syntax_id: child_id,
                    })?;
            current_child_index += 1;

            if child_node.kind.is_statement() {
                self.bind_statement(child_id, &child_node)?;
            } else if child_node.kind.is_item() {
                self.bind_item(child_id, &child_node)?;
            } else if child_node.kind.is_expression() {
                self.bind_expression(child_id, &child_node)?;
            }
        }

        Ok(())
    }

    fn bind_statement(&mut self, node_id: SyntaxId, node: &SyntaxNode) -> Result<(), CompileError> {
        match node.kind {
            SyntaxKind::ExpressionStatement => self.bind_expression_statement(node_id, node),
            SyntaxKind::LetStatement | SyntaxKind::LetMutStatement => self.bind_let_statement(node),
            SyntaxKind::ReassignmentStatement => self.bind_reassignment_statement(node),
            SyntaxKind::AdditionAssignmentStatement
            | SyntaxKind::SubtractionAssignmentStatement
            | SyntaxKind::MultiplicationAssignmentStatement
            | SyntaxKind::DivisionAssignmentStatement
            | SyntaxKind::ModuloAssignmentStatement
            | SyntaxKind::ExponentAssignmentStatement => self.bind_assignment_statement(node),
            SyntaxKind::SemicolonStatement => Ok(()),
            _ => Ok(()),
        }
    }

    fn bind_expression_statement(
        &mut self,
        _node_id: SyntaxId,
        node: &SyntaxNode,
    ) -> Result<(), CompileError> {
        info!("Binding expression statement");

        let child_id = SyntaxId(node.children.0);
        let child_node =
            *self
                .syntax_tree
                .get_node(child_id)
                .ok_or(CompileError::MissingSyntaxNode {
                    syntax_id: child_id,
                })?;

        if child_node.kind.is_expression() {
            self.bind_expression(child_id, &child_node)
        } else if child_node.kind.is_statement() {
            self.bind_statement(child_id, &child_node)
        } else {
            Ok(())
        }
    }

    fn bind_let_statement(&mut self, node: &SyntaxNode) -> Result<(), CompileError> {
        info!("Binding let statement");

        let path_id = SyntaxId(node.children.0);
        let expression_statement_id = SyntaxId(node.children.1);
        let expression_statement = *self.syntax_tree.get_node(expression_statement_id).ok_or(
            CompileError::MissingSyntaxNode {
                syntax_id: expression_statement_id,
            },
        )?;
        let expression_id = SyntaxId(expression_statement.children.0);
        let expression =
            *self
                .syntax_tree
                .get_node(expression_id)
                .ok_or(CompileError::MissingSyntaxNode {
                    syntax_id: SyntaxId(expression_statement.children.0),
                })?;

        self.bind_expression(expression_id, &expression)?;

        let path_node = *self
            .syntax_tree
            .get_node(path_id)
            .ok_or(CompileError::MissingSyntaxNode { syntax_id: path_id })?;
        let files = self.source.read_files();
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
            .resolver
            .find_declaration_in_scope(variable_name, self.current_scope_id)
            .map(|(id, _)| id);
        let declaration_kind = if node.kind == SyntaxKind::LetStatement {
            DeclarationKind::Local { shadowed }
        } else {
            DeclarationKind::LocalMutable { shadowed }
        };

        let local_type_id = self.resolver.create_inferred_type();

        self.resolver.add_declaration(
            variable_name,
            Declaration {
                kind: declaration_kind,
                scope_id: self.current_scope_id,
                type_id: local_type_id,
                position: Position::new(self.file_id, node.span),
                is_public: false,
            },
        );

        Ok(())
    }

    fn bind_reassignment_statement(&mut self, node: &SyntaxNode) -> Result<(), CompileError> {
        info!("Binding reassignment statement");

        let expression_statement_id = SyntaxId(node.children.1);
        let expression_statement_node = *self.syntax_tree.get_node(expression_statement_id).ok_or(
            CompileError::MissingSyntaxNode {
                syntax_id: expression_statement_id,
            },
        )?;
        let expression_id = SyntaxId(expression_statement_node.children.0);
        let expression_node =
            *self
                .syntax_tree
                .get_node(expression_id)
                .ok_or(CompileError::MissingSyntaxNode {
                    syntax_id: expression_id,
                })?;

        self.bind_expression(expression_id, &expression_node)?;

        Ok(())
    }

    fn bind_assignment_statement(&mut self, node: &SyntaxNode) -> Result<(), CompileError> {
        info!("Binding assignment statement");

        let left_id = SyntaxId(node.children.0);
        let left_node = self
            .syntax_tree
            .get_node(left_id)
            .ok_or(CompileError::MissingSyntaxNode { syntax_id: left_id })?;

        self.bind_expression(left_id, left_node)?;

        let right_id = SyntaxId(node.children.1);
        let right_node =
            self.syntax_tree
                .get_node(right_id)
                .ok_or(CompileError::MissingSyntaxNode {
                    syntax_id: right_id,
                })?;

        self.bind_expression(right_id, right_node)?;

        Ok(())
    }

    fn bind_expression(
        &mut self,
        node_id: SyntaxId,
        node: &SyntaxNode,
    ) -> Result<(), CompileError> {
        match node.kind {
            SyntaxKind::BlockExpression => self.bind_block_expression(node_id, node),
            SyntaxKind::FunctionExpression => self.bind_function_expression(node),
            SyntaxKind::WhileExpression => self.bind_while_expression(node),
            SyntaxKind::IfExpression => self.bind_if_expression(node),
            SyntaxKind::CallExpression => self.bind_call_expression(node),
            SyntaxKind::ListExpression => self.bind_list_expression(node),
            SyntaxKind::IndexExpression => self.bind_index_expression(node),
            SyntaxKind::GroupedExpression => self.bind_grouped_expression(node),
            SyntaxKind::AdditionExpression
            | SyntaxKind::SubtractionExpression
            | SyntaxKind::MultiplicationExpression
            | SyntaxKind::DivisionExpression
            | SyntaxKind::ModuloExpression
            | SyntaxKind::ExponentExpression
            | SyntaxKind::GreaterThanExpression
            | SyntaxKind::GreaterThanOrEqualExpression
            | SyntaxKind::LessThanExpression
            | SyntaxKind::LessThanOrEqualExpression
            | SyntaxKind::EqualExpression
            | SyntaxKind::NotEqualExpression
            | SyntaxKind::AndExpression
            | SyntaxKind::OrExpression => self.bind_binary_expression(node),
            SyntaxKind::NotExpression | SyntaxKind::NegationExpression => {
                self.bind_unary_expression(node)
            }
            SyntaxKind::AsExpression => self.bind_as_expression(node),
            SyntaxKind::ElseExpression => self.bind_else_expression(node),
            // Literal and path expressions don't create scopes or declarations
            SyntaxKind::BooleanExpression
            | SyntaxKind::ByteExpression
            | SyntaxKind::CharacterExpression
            | SyntaxKind::FloatExpression
            | SyntaxKind::IntegerExpression
            | SyntaxKind::StringExpression
            | SyntaxKind::PathExpression
            | SyntaxKind::ReturnExpression
            | SyntaxKind::BreakExpression => Ok(()),
            _ => Ok(()),
        }
    }

    fn bind_block_expression(
        &mut self,
        syntax_id: SyntaxId,
        node: &SyntaxNode,
    ) -> Result<(), CompileError> {
        info!("Binding block expression");

        let start_children = node.children.0 as usize;
        let child_count = node.children.1 as usize;

        if child_count == 0 {
            return Ok(());
        }

        let parent_scope_id = self.current_scope_id;
        let block_scope_id = self.resolver.add_scope(Scope {
            kind: ScopeKind::Block,
            parent: parent_scope_id,
            imports: SmallVec::new(),
            modules: SmallVec::new(),
        });

        self.resolver.add_scope_binding(syntax_id, block_scope_id);

        self.current_scope_id = block_scope_id;

        let end_children = start_children + child_count;
        let mut current_child_index = start_children;

        while current_child_index < end_children {
            let child_id = *self.syntax_tree.children.get(current_child_index).ok_or(
                CompileError::MissingChild {
                    parent_kind: node.kind,
                    child_index: current_child_index as u32,
                },
            )?;
            let child_node =
                self.syntax_tree
                    .get_node(child_id)
                    .ok_or(CompileError::MissingSyntaxNode {
                        syntax_id: child_id,
                    })?;
            current_child_index += 1;

            if child_node.kind.is_statement() {
                self.bind_statement(child_id, child_node)?;
            } else if child_node.kind.is_item() {
                self.bind_item(child_id, child_node)?;
            } else if child_node.kind.is_expression() {
                self.bind_expression(child_id, child_node)?;
            }
        }

        self.current_scope_id = parent_scope_id;

        Ok(())
    }

    fn bind_function_expression(&mut self, node: &SyntaxNode) -> Result<(), CompileError> {
        info!("Binding function expression");

        let function_signature_id = SyntaxId(node.children.0);
        let function_signature_node = *self.syntax_tree.get_node(function_signature_id).ok_or(
            CompileError::MissingSyntaxNode {
                syntax_id: function_signature_id,
            },
        )?;

        debug_assert_eq!(function_signature_node.kind, SyntaxKind::FunctionSignature);

        let value_parameter_list_id = SyntaxId(function_signature_node.children.0);
        let value_parameter_list_node = *self.syntax_tree.get_node(value_parameter_list_id).ok_or(
            CompileError::MissingSyntaxNode {
                syntax_id: value_parameter_list_id,
            },
        )?;

        debug_assert_eq!(
            value_parameter_list_node.kind,
            SyntaxKind::ValueParametersDefinition
        );

        let function_scope = self.resolver.add_scope(Scope {
            kind: ScopeKind::Function,
            parent: self.current_scope_id,
            imports: SmallVec::new(),
            modules: SmallVec::new(),
        });

        let value_parameter_node_ids = self
            .syntax_tree
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
                self.syntax_tree
                    .get_node(id)
                    .ok_or(CompileError::MissingSyntaxNode { syntax_id: id })
                    .copied()
            })
            .collect::<Result<SmallVec<[SyntaxNode; 4]>, CompileError>>()?;

        let value_parameters_range = {
            let files = self.source.read_files();
            let file =
                files
                    .get(self.file_id.0 as usize)
                    .ok_or(CompileError::MissingSourceFile {
                        file_id: self.file_id,
                    })?;

            let mut value_parameters = SmallVec::<[TypeId; 8]>::new();
            let mut current_parameter_name = "";

            for (index, param_node) in value_parameter_nodes.iter().enumerate() {
                let is_name = index % 2 == 0;

                if is_name {
                    current_parameter_name = unsafe {
                        str::from_utf8_unchecked(
                            &file.source_code.as_ref()
                                [param_node.span.0 as usize..param_node.span.1 as usize],
                        )
                    };
                } else {
                    let type_id = Self::get_type_id(param_node, self.syntax_tree, self.resolver)?;
                    let parameter_declaration = Declaration {
                        kind: DeclarationKind::Local { shadowed: None },
                        scope_id: function_scope,
                        type_id,
                        position: Position::new(self.file_id, param_node.span),
                        is_public: false,
                    };

                    self.resolver
                        .add_declaration(current_parameter_name, parameter_declaration);
                    value_parameters.push(type_id);
                }
            }

            self.resolver.add_type_members(&value_parameters)
        };

        let return_type_node_id = SyntaxId(function_signature_node.children.1);
        let return_type_id = if return_type_node_id == SyntaxId::NONE {
            TypeId::NONE
        } else {
            let function_return_type_node = *self.syntax_tree.get_node(return_type_node_id).ok_or(
                CompileError::MissingSyntaxNode {
                    syntax_id: return_type_node_id,
                },
            )?;

            Self::get_type_id(&function_return_type_node, self.syntax_tree, self.resolver)?
        };

        let _function_type_node = FunctionTypeNode {
            value_parameters: value_parameters_range,
            type_parameters: (0, 0),
            return_type: return_type_id,
        };

        let body_id = SyntaxId(node.children.1);
        let body_node = *self
            .syntax_tree
            .get_node(body_id)
            .ok_or(CompileError::MissingSyntaxNode { syntax_id: body_id })?;

        let parent_scope_id = self.current_scope_id;
        self.current_scope_id = function_scope;

        self.resolver.add_scope_binding(body_id, function_scope);
        self.bind_function(body_id, &body_node)?;

        self.current_scope_id = parent_scope_id;

        Ok(())
    }

    fn bind_while_expression(&mut self, node: &SyntaxNode) -> Result<(), CompileError> {
        info!("Binding while expression");

        let condition_id = SyntaxId(node.children.0);
        let condition_node =
            self.syntax_tree
                .get_node(condition_id)
                .ok_or(CompileError::MissingSyntaxNode {
                    syntax_id: condition_id,
                })?;

        self.bind_expression(condition_id, condition_node)?;

        let body_id = SyntaxId(node.children.1);
        let body_node = self
            .syntax_tree
            .get_node(body_id)
            .ok_or(CompileError::MissingSyntaxNode { syntax_id: body_id })?;

        if body_node.kind == SyntaxKind::ExpressionStatement {
            self.bind_expression_statement(body_id, body_node)?;
        } else {
            self.bind_expression(body_id, body_node)?;
        }

        Ok(())
    }

    fn bind_if_expression(&mut self, node: &SyntaxNode) -> Result<(), CompileError> {
        info!("Binding if expression");

        let child_ids = self
            .syntax_tree
            .get_children(node.children.0, node.children.1)
            .ok_or(CompileError::MissingChildren {
                parent_kind: node.kind,
                start_index: node.children.0,
                count: node.children.1,
            })?
            .iter()
            .cloned()
            .collect::<SmallVec<[SyntaxId; 3]>>();

        let condition_id = *child_ids.first().ok_or(CompileError::MissingChild {
            parent_kind: node.kind,
            child_index: 0,
        })?;
        let condition_node =
            self.syntax_tree
                .get_node(condition_id)
                .ok_or(CompileError::MissingSyntaxNode {
                    syntax_id: condition_id,
                })?;

        self.bind_expression(condition_id, condition_node)?;

        let then_block_id = *child_ids.get(1).ok_or(CompileError::MissingChild {
            parent_kind: node.kind,
            child_index: 1,
        })?;
        let then_block_node =
            self.syntax_tree
                .get_node(then_block_id)
                .ok_or(CompileError::MissingSyntaxNode {
                    syntax_id: then_block_id,
                })?;

        match then_block_node.kind {
            SyntaxKind::BlockExpression => {
                self.bind_block_expression(then_block_id, then_block_node)?;
            }
            SyntaxKind::ExpressionStatement => {
                self.bind_expression_statement(then_block_id, then_block_node)?;
            }
            _ => {
                self.bind_expression(then_block_id, then_block_node)?;
            }
        }

        if child_ids.len() == 3 {
            let else_block_id = *child_ids.get(2).ok_or(CompileError::MissingChild {
                parent_kind: node.kind,
                child_index: 2,
            })?;
            let else_block_node = self.syntax_tree.get_node(else_block_id).ok_or(
                CompileError::MissingSyntaxNode {
                    syntax_id: else_block_id,
                },
            )?;

            match else_block_node.kind {
                SyntaxKind::IfExpression => self.bind_if_expression(else_block_node)?,
                SyntaxKind::BlockExpression => {
                    self.bind_block_expression(else_block_id, else_block_node)?
                }
                SyntaxKind::ExpressionStatement => {
                    self.bind_expression_statement(else_block_id, else_block_node)?
                }
                _ => self.bind_expression(else_block_id, else_block_node)?,
            }
        }

        Ok(())
    }

    fn bind_else_expression(&mut self, node: &SyntaxNode) -> Result<(), CompileError> {
        info!("Binding else expression");

        let child_id = SyntaxId(node.children.0);
        let child_node =
            self.syntax_tree
                .get_node(child_id)
                .ok_or(CompileError::MissingSyntaxNode {
                    syntax_id: child_id,
                })?;

        match child_node.kind {
            SyntaxKind::IfExpression => self.bind_if_expression(child_node),
            SyntaxKind::BlockExpression => self.bind_block_expression(child_id, child_node),
            SyntaxKind::ExpressionStatement => self.bind_expression_statement(child_id, child_node),
            _ => self.bind_expression(child_id, child_node),
        }
    }

    fn bind_call_expression(&mut self, node: &SyntaxNode) -> Result<(), CompileError> {
        info!("Binding call expression");

        let function_node_id = SyntaxId(node.children.0);
        let function_node =
            self.syntax_tree
                .get_node(function_node_id)
                .ok_or(CompileError::MissingSyntaxNode {
                    syntax_id: function_node_id,
                })?;

        self.bind_expression(function_node_id, function_node)?;

        let arguments_node_id = SyntaxId(node.children.1);
        let arguments_node = *self.syntax_tree.get_node(arguments_node_id).ok_or(
            CompileError::MissingSyntaxNode {
                syntax_id: arguments_node_id,
            },
        )?;

        if arguments_node.kind == SyntaxKind::CallValueArguments {
            let (start_children, child_count) = (
                arguments_node.children.0 as usize,
                arguments_node.children.1 as usize,
            );
            let end_children = start_children + child_count;
            let mut current_child_index = start_children;

            while current_child_index < end_children {
                let argument_node_id = *self.syntax_tree.children.get(current_child_index).ok_or(
                    CompileError::MissingChild {
                        parent_kind: arguments_node.kind,
                        child_index: current_child_index as u32,
                    },
                )?;
                let argument_node = self.syntax_tree.get_node(argument_node_id).ok_or(
                    CompileError::MissingSyntaxNode {
                        syntax_id: argument_node_id,
                    },
                )?;
                current_child_index += 1;

                self.bind_expression(argument_node_id, argument_node)?;
            }
        }

        Ok(())
    }

    fn bind_list_expression(&mut self, node: &SyntaxNode) -> Result<(), CompileError> {
        info!("Binding list expression");

        let (start_children, child_count) = (node.children.0 as usize, node.children.1 as usize);
        let mut current_child_index = start_children;

        for _ in 0..child_count {
            let child_id = *self.syntax_tree.children.get(current_child_index).ok_or(
                CompileError::MissingChild {
                    parent_kind: node.kind,
                    child_index: current_child_index as u32,
                },
            )?;
            let child_node =
                self.syntax_tree
                    .get_node(child_id)
                    .ok_or(CompileError::MissingSyntaxNode {
                        syntax_id: child_id,
                    })?;
            current_child_index += 1;

            self.bind_expression(child_id, child_node)?;
        }

        Ok(())
    }

    fn bind_index_expression(&mut self, node: &SyntaxNode) -> Result<(), CompileError> {
        info!("Binding index expression");

        let list_id = SyntaxId(node.children.0);
        let list_node = self
            .syntax_tree
            .get_node(list_id)
            .ok_or(CompileError::MissingSyntaxNode { syntax_id: list_id })?;

        self.bind_expression(list_id, list_node)?;

        let index_id = SyntaxId(node.children.1);
        let index_node =
            self.syntax_tree
                .get_node(index_id)
                .ok_or(CompileError::MissingSyntaxNode {
                    syntax_id: index_id,
                })?;

        self.bind_expression(index_id, index_node)?;

        Ok(())
    }

    fn bind_grouped_expression(&mut self, node: &SyntaxNode) -> Result<(), CompileError> {
        info!("Binding grouped expression");

        let child_id = SyntaxId(node.children.0);
        let child_node =
            self.syntax_tree
                .get_node(child_id)
                .ok_or(CompileError::MissingSyntaxNode {
                    syntax_id: child_id,
                })?;

        self.bind_expression(child_id, child_node)
    }

    fn bind_binary_expression(&mut self, node: &SyntaxNode) -> Result<(), CompileError> {
        info!("Binding binary expression");

        let left_id = SyntaxId(node.children.0);
        let left_node = self
            .syntax_tree
            .get_node(left_id)
            .ok_or(CompileError::MissingSyntaxNode { syntax_id: left_id })?;

        self.bind_expression(left_id, left_node)?;

        let right_id = SyntaxId(node.children.1);
        let right_node =
            self.syntax_tree
                .get_node(right_id)
                .ok_or(CompileError::MissingSyntaxNode {
                    syntax_id: right_id,
                })?;

        self.bind_expression(right_id, right_node)?;

        Ok(())
    }

    fn bind_unary_expression(&mut self, node: &SyntaxNode) -> Result<(), CompileError> {
        info!("Binding unary expression");

        let operand_id = SyntaxId(node.children.0);
        let operand_node =
            self.syntax_tree
                .get_node(operand_id)
                .ok_or(CompileError::MissingSyntaxNode {
                    syntax_id: operand_id,
                })?;

        self.bind_expression(operand_id, operand_node)
    }

    fn bind_as_expression(&mut self, node: &SyntaxNode) -> Result<(), CompileError> {
        info!("Binding as expression");

        let expression_id = SyntaxId(node.children.0);
        let expression_node =
            self.syntax_tree
                .get_node(expression_id)
                .ok_or(CompileError::MissingSyntaxNode {
                    syntax_id: expression_id,
                })?;

        self.bind_expression(expression_id, expression_node)
    }

    fn get_type_id(
        node: &SyntaxNode,
        syntax_tree: &SyntaxTree,
        resolver: &mut Resolver,
    ) -> Result<TypeId, CompileError> {
        match node.kind {
            SyntaxKind::BooleanType => Ok(TypeId::BOOLEAN),
            SyntaxKind::ByteType => Ok(TypeId::BYTE),
            SyntaxKind::CharacterType => Ok(TypeId::CHARACTER),
            SyntaxKind::FloatType => Ok(TypeId::FLOAT),
            SyntaxKind::IntegerType => Ok(TypeId::INTEGER),
            SyntaxKind::StringType => Ok(TypeId::STRING),
            SyntaxKind::ListType => {
                let element_type_id = SyntaxId(node.children.0);
                let element_type_node = syntax_tree.get_node(element_type_id).ok_or(
                    CompileError::MissingSyntaxNode {
                        syntax_id: element_type_id,
                    },
                )?;

                let element_type_id = Self::get_type_id(element_type_node, syntax_tree, resolver)?;
                let lise_type_id = resolver.add_type_node(TypeNode::List(element_type_id));

                Ok(lise_type_id)
            }
            SyntaxKind::FunctionType => {
                let function_value_parameters_id = SyntaxId(node.children.0);
                let type_node_value_parameters = if function_value_parameters_id == SyntaxId::NONE {
                    (0, 0)
                } else {
                    let function_value_parameters_node = syntax_tree
                        .get_node(function_value_parameters_id)
                        .ok_or(CompileError::MissingSyntaxNode {
                            syntax_id: function_value_parameters_id,
                        })?;

                    let value_parameter_type_node_ids = syntax_tree
                        .get_children(
                            function_value_parameters_node.children.0,
                            function_value_parameters_node.children.1,
                        )
                        .ok_or(CompileError::MissingChildren {
                            parent_kind: function_value_parameters_node.kind,
                            start_index: function_value_parameters_node.children.0,
                            count: function_value_parameters_node.children.1,
                        })?;

                    let mut value_parameter_type_ids =
                        SmallVec::<[TypeId; 4]>::with_capacity(value_parameter_type_node_ids.len());

                    for type_node_id in value_parameter_type_node_ids {
                        let type_id = if *type_node_id == SyntaxId::NONE {
                            TypeId::NONE
                        } else {
                            let type_node = syntax_tree.get_node(*type_node_id).ok_or(
                                CompileError::MissingSyntaxNode {
                                    syntax_id: *type_node_id,
                                },
                            )?;

                            Self::get_type_id(type_node, syntax_tree, resolver)?
                        };

                        value_parameter_type_ids.push(type_id);
                    }

                    resolver.add_type_members(&value_parameter_type_ids)
                };

                let function_return_type_id = SyntaxId(node.children.1);
                let return_type_id = if function_return_type_id == SyntaxId::NONE {
                    TypeId::NONE
                } else {
                    let function_return_type_node = syntax_tree
                        .get_node(function_return_type_id)
                        .ok_or(CompileError::MissingSyntaxNode {
                        syntax_id: function_return_type_id,
                    })?;

                    Self::get_type_id(function_return_type_node, syntax_tree, resolver)?
                };

                let function_type_node = FunctionTypeNode {
                    type_parameters: (0, 0),
                    value_parameters: type_node_value_parameters,
                    return_type: return_type_id,
                };
                let function_type_id =
                    resolver.add_type_node(TypeNode::Function(function_type_node));

                Ok(function_type_id)
            }
            _ => {
                todo!()
            }
        }
    }
}
