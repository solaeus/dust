use smallvec::SmallVec;
use tracing::{debug, info};

use crate::{
    compiler::{
        CompileError,
        context::{
            CompileContext, Declaration, DeclarationId, DeclarationKind, Scope, ScopeId, ScopeKind,
        },
    },
    source::{Position, Source, SourceFileId},
    syntax::{Syntax, SyntaxId, SyntaxKind, SyntaxReader, SyntaxVisitor},
};

pub struct DeclarationBinder<'a> {
    function_id: Option<DeclarationId>,

    file_id: SourceFileId,

    source: &'a Source,

    syntax: &'a Syntax,

    context: &'a mut CompileContext,

    current_scope_id: ScopeId,
}

impl<'a> DeclarationBinder<'a> {
    pub fn new(
        declaration_id: Option<DeclarationId>,
        file_id: SourceFileId,
        source: &'a Source,
        syntax: &'a Syntax,
        context: &'a mut CompileContext,
        current_scope_id: ScopeId,
    ) -> Self {
        Self {
            function_id: declaration_id,
            file_id,
            source,
            syntax,
            context,
            current_scope_id,
        }
    }

    pub fn bind_main(mut self) -> Result<(), CompileError> {
        let main_root = self
            .syntax
            .get_tree(SourceFileId::MAIN)
            .ok_or(CompileError::MissingSyntaxTree {
                file_id: SourceFileId::MAIN,
            })?
            .root()
            .ok_or(CompileError::MissingSyntaxNode {
                syntax_id: SyntaxId::ROOT,
            })?;

        self.visit_main_function_item(main_root, ())
    }
}

impl<'a> SyntaxVisitor for DeclarationBinder<'a> {
    type Input = ();

    type Output = ();

    fn file_id(&self) -> SourceFileId {
        self.file_id
    }

    fn visit_main_function_item(
        &mut self,
        node: SyntaxReader,
        _: Self::Input,
    ) -> Result<Self::Output, CompileError> {
        debug!("Binding main function");

        self.context.add_scope(Scope {
            kind: ScopeKind::Function,
            parent: ScopeId::PROJECT,
            imports: SmallVec::new(),
            modules: SmallVec::new(),
        });

        let children = node
            .multiple_children()
            .ok_or(CompileError::MissingChildren {
                parent_kind: SyntaxKind::MainFunctionItem,
                start_index: node.inner().children.0,
                count: node.inner().children.1,
            })?;

        for child in children {
            self.visit(child, ())?;
        }

        Ok(())
    }

    fn visit_module_item(
        &mut self,
        _: SyntaxReader,
        _: Self::Input,
    ) -> Result<Self::Output, CompileError> {
        debug!("Binding module item");

        todo!()
    }

    fn visit_function_item(
        &mut self,
        node: SyntaxReader,
        _: Self::Input,
    ) -> Result<Self::Output, CompileError> {
        debug!("Binding function item");

        let function_name = node.left_child().ok_or(CompileError::MissingChild {
            parent_kind: node.kind(),
            child_index: 0,
        })?;
        let function_expression = node.right_child().ok_or(CompileError::MissingChild {
            parent_kind: node.kind(),
            child_index: 1,
        })?;
        let signature = function_expression
            .left_child()
            .ok_or(CompileError::MissingChild {
                parent_kind: function_expression.kind(),
                child_index: 0,
            })?;
        let value_parameters = signature
            .left_child()
            .ok_or(CompileError::MissingChild {
                parent_kind: signature.kind(),
                child_index: 0,
            })?
            .multiple_children()
            .ok_or(CompileError::MissingChildren {
                parent_kind: signature.kind(),
                start_index: signature.inner().children.0,
                count: signature.inner().children.1,
            })?;
        let function_body =
            function_expression
                .right_child()
                .ok_or(CompileError::MissingChild {
                    parent_kind: function_expression.kind(),
                    child_index: 1,
                })?;

        let function_scope_id = self.context.add_scope(Scope {
            kind: ScopeKind::Function,
            parent: self.current_scope_id,
            imports: SmallVec::new(),
            modules: SmallVec::new(),
        });

        let mut parameter_ids = SmallVec::<[DeclarationId; 8]>::new();

        for value_parameter in value_parameters {
            let parameter_name =
                value_parameter
                    .left_child()
                    .ok_or(CompileError::MissingChild {
                        parent_kind: value_parameter.kind(),
                        child_index: 0,
                    })?;

            let source_file = self.source.files().get(self.file_id.0 as usize).ok_or(
                CompileError::MissingSourceFile {
                    file_id: self.file_id,
                },
            )?;
            let parameter_name_str = source_file.source_code.get_span(parameter_name.span());

            let parameter_declaration = Declaration {
                kind: DeclarationKind::Local { shadowed: None },
                scope_id: function_scope_id,
                position: Position::new(self.file_id, parameter_name.span()),
                is_public: false,
            };
            let parameter_declaration_id = self
                .context
                .add_declaration(parameter_name_str, parameter_declaration);

            self.context
                .set_declaration_binding(parameter_name.id, parameter_declaration_id);
            parameter_ids.push(parameter_declaration_id);
        }

        let is_public = match node.kind() {
            SyntaxKind::PublicFunctionItem => true,
            SyntaxKind::FunctionItem => false,
            _ => unreachable!(),
        };
        let parameters = self.context.add_declaration_members(&parameter_ids);
        let function_declaration = Declaration {
            kind: DeclarationKind::Function {
                file_id: self.file_id,
                syntax_id: node.id,
                parameters,
                prototype_index: None,
            },
            scope_id: self.current_scope_id,
            position: Position::new(self.file_id, function_name.span()),
            is_public,
        };

        let source_file = self.source.files().get(self.file_id.0 as usize).ok_or(
            CompileError::MissingSourceFile {
                file_id: self.file_id,
            },
        )?;
        let function_name_str = source_file.source_code.get_span(function_name.span());
        let function_declaration_id = self
            .context
            .add_declaration(function_name_str, function_declaration);

        self.context
            .add_scope_binding(function_body.id, function_scope_id);
        self.context
            .set_declaration_binding(function_expression.id, function_declaration_id);

        let mut function_declaration_binder = DeclarationBinder::new(
            Some(function_declaration_id),
            self.file_id,
            self.source,
            self.syntax,
            self.context,
            function_scope_id,
        );

        function_declaration_binder.visit(function_body, ())?;

        Ok(())
    }

    fn visit_use_item(
        &mut self,
        _: SyntaxReader,
        _: Self::Input,
    ) -> Result<Self::Output, CompileError> {
        debug!("Binding use item");

        todo!()
    }

    fn visit_expression_statement(
        &mut self,
        node: SyntaxReader,
        _: Self::Input,
    ) -> Result<Self::Output, CompileError> {
        debug!("Binding expression statement");

        let expression = node.left_child().ok_or(CompileError::MissingChild {
            parent_kind: node.kind(),
            child_index: 0,
        })?;

        self.visit_expression(expression, ())?;

        Ok(())
    }

    fn visit_let_statement(
        &mut self,
        node: SyntaxReader,
        _: Self::Input,
    ) -> Result<Self::Output, CompileError> {
        info!("Binding let statement");

        let mut children = node
            .multiple_children()
            .ok_or(CompileError::MissingChildren {
                parent_kind: node.inner().kind,
                start_index: node.inner().children.0,
                count: node.inner().children.1,
            })?;
        let path = children.next().ok_or(CompileError::MissingChild {
            parent_kind: node.inner().kind,
            child_index: 0,
        })?;
        let expression_statement = children.next().ok_or(CompileError::MissingChild {
            parent_kind: node.inner().kind,
            child_index: 1,
        })?;
        let expression = expression_statement
            .left_child()
            .ok_or(CompileError::MissingChild {
                parent_kind: expression_statement.kind(),
                child_index: 0,
            })?;

        self.visit_expression(expression, ())?;

        let source_file = self.source.files().get(self.file_id.0 as usize).ok_or(
            CompileError::MissingSourceFile {
                file_id: self.file_id,
            },
        )?;
        let variable_name = source_file
            .source_code
            .get(path.span().0 as usize, path.span().1 as usize);

        let shadowed = self
            .context
            .find_declaration_in_scope(variable_name, self.current_scope_id)
            .map(|(id, _)| id);
        let declaration_kind = if node.kind() == SyntaxKind::LetMutStatement {
            DeclarationKind::LocalMutable { shadowed }
        } else {
            DeclarationKind::Local { shadowed }
        };
        let declaration = Declaration {
            kind: declaration_kind,
            scope_id: self.current_scope_id,
            position: Position::new(self.file_id, path.span()),
            is_public: false,
        };
        let declaration_id = self.context.add_declaration(variable_name, declaration);

        self.context
            .set_declaration_binding(path.id, declaration_id);

        Ok(())
    }

    fn visit_binary_assignment_statement(
        &mut self,
        node: SyntaxReader,
        input: Self::Input,
    ) -> Result<Self::Output, CompileError> {
        info!("Binding binary assignment statement");

        let path = node.left_child().ok_or(CompileError::MissingChild {
            parent_kind: node.kind(),
            child_index: 0,
        })?;
        let expression = node.right_child().ok_or(CompileError::MissingChild {
            parent_kind: node.kind(),
            child_index: 1,
        })?;

        self.visit_expression(expression, input)?;

        let source_file = self.source.files().get(self.file_id.0 as usize).ok_or(
            CompileError::MissingSourceFile {
                file_id: self.file_id,
            },
        )?;
        let variable_name = source_file
            .source_code
            .get(path.span().0 as usize, path.span().1 as usize);

        let (declaration_id, _) = self
            .context
            .find_declaration_in_scope(variable_name, self.current_scope_id)
            .ok_or(CompileError::UndeclaredVariable {
                name: variable_name.to_string(),
                position: Position::new(self.file_id, path.span()),
            })?;

        self.context
            .set_declaration_binding(path.id, declaration_id);

        Ok(())
    }

    fn visit_reassignment_statement(
        &mut self,
        node: SyntaxReader,
        _: Self::Input,
    ) -> Result<Self::Output, CompileError> {
        let path = node.left_child().ok_or(CompileError::MissingChild {
            parent_kind: node.kind(),
            child_index: 0,
        })?;
        let expression_statement = node.right_child().ok_or(CompileError::MissingChild {
            parent_kind: node.kind(),
            child_index: 1,
        })?;
        let expression = expression_statement
            .left_child()
            .ok_or(CompileError::MissingChild {
                parent_kind: expression_statement.kind(),
                child_index: 0,
            })?;

        let source_file = self.source.files().get(self.file_id.0 as usize).ok_or(
            CompileError::MissingSourceFile {
                file_id: self.file_id,
            },
        )?;
        let variable_name = source_file.source_code.get_span(path.span());

        let (declaration_id, _) = self
            .context
            .find_declaration_in_scope(variable_name, self.current_scope_id)
            .ok_or(CompileError::UndeclaredVariable {
                name: variable_name.to_string(),
                position: Position::new(self.file_id, path.span()),
            })?;

        self.context
            .set_declaration_binding(path.id, declaration_id);
        self.visit_expression(expression, ())?;

        Ok(())
    }

    fn visit_boolean_expression(
        &mut self,
        _: SyntaxReader,
        _: Self::Input,
    ) -> Result<Self::Output, CompileError> {
        Ok(())
    }

    fn visit_byte_expression(
        &mut self,
        _: SyntaxReader,
        _: Self::Input,
    ) -> Result<Self::Output, CompileError> {
        Ok(())
    }

    fn visit_character_expression(
        &mut self,
        _: SyntaxReader,
        _: Self::Input,
    ) -> Result<Self::Output, CompileError> {
        Ok(())
    }

    fn visit_float_expression(
        &mut self,
        _: SyntaxReader,
        _: Self::Input,
    ) -> Result<Self::Output, CompileError> {
        Ok(())
    }

    fn visit_integer_expression(
        &mut self,
        _: SyntaxReader,
        _: Self::Input,
    ) -> Result<Self::Output, CompileError> {
        Ok(())
    }

    fn visit_string_expression(
        &mut self,
        _: SyntaxReader,
        _: Self::Input,
    ) -> Result<Self::Output, CompileError> {
        Ok(())
    }

    fn visit_list_expression(
        &mut self,
        node: SyntaxReader,
        _: Self::Input,
    ) -> Result<Self::Output, CompileError> {
        debug!("Binding list expression");

        let elements = node
            .multiple_children()
            .ok_or(CompileError::MissingChildren {
                parent_kind: node.inner().kind,
                start_index: node.inner().children.0,
                count: node.inner().children.1,
            })?;

        for element in elements {
            self.visit_expression(element, ())?;
        }

        Ok(())
    }

    fn visit_index_expression(
        &mut self,
        node: SyntaxReader,
        _: Self::Input,
    ) -> Result<Self::Output, CompileError> {
        debug!("Binding index expression");

        let list = node.left_child().ok_or(CompileError::MissingChild {
            parent_kind: node.kind(),
            child_index: 0,
        })?;
        let index = node.right_child().ok_or(CompileError::MissingChild {
            parent_kind: node.kind(),
            child_index: 1,
        })?;

        self.visit_expression(list, ())?;
        self.visit_expression(index, ())?;

        Ok(())
    }

    fn visit_path_expression(
        &mut self,
        node: SyntaxReader,
        _: Self::Input,
    ) -> Result<Self::Output, CompileError> {
        debug!("Binding path expression");

        let path = node.left_child().ok_or(CompileError::MissingChild {
            parent_kind: node.kind(),
            child_index: 0,
        })?;
        let path_segments = path
            .multiple_children()
            .ok_or(CompileError::MissingChildren {
                parent_kind: path.kind(),
                start_index: path.inner().children.0,
                count: path.inner().children.1,
            })?;

        let mut current_declaration_id = DeclarationId(0);
        let mut current_scope_id = self.current_scope_id;

        for segment in path_segments {
            let source_file = self.source.files().get(self.file_id.0 as usize).ok_or(
                CompileError::MissingSourceFile {
                    file_id: self.file_id,
                },
            )?;
            let segment_name = source_file.source_code.get_span(segment.span());
            let (next_declaration_id, next_declaration) = self
                .context
                .find_declaration_in_scope(segment_name, current_scope_id)
                .or_else(|| {
                    if let Some(id) = self.function_id {
                        let declaration = *self.context.get_declaration(id)?;

                        Some((id, declaration))
                    } else {
                        None
                    }
                })
                .ok_or(CompileError::UndeclaredVariable {
                    name: segment_name.to_string(),
                    position: Position::new(self.file_id, segment.span()),
                })?;

            current_declaration_id = next_declaration_id;
            current_scope_id = next_declaration.scope_id;
        }

        self.context
            .set_declaration_binding(node.id, current_declaration_id);

        Ok(())
    }

    fn visit_block_expression(
        &mut self,
        node: SyntaxReader,
        _: Self::Input,
    ) -> Result<Self::Output, CompileError> {
        debug!("Binding block expression");

        let children = node
            .multiple_children()
            .ok_or(CompileError::MissingChildren {
                parent_kind: node.inner().kind,
                start_index: node.inner().children.0,
                count: node.inner().children.1,
            })?;

        let block_scope_id = self.context.add_scope(Scope {
            kind: ScopeKind::Block,
            parent: self.current_scope_id,
            imports: SmallVec::new(),
            modules: SmallVec::new(),
        });
        let parent_scope_id = self.current_scope_id;
        self.current_scope_id = block_scope_id;

        for child in children {
            self.visit(child, ())?;
        }

        self.current_scope_id = parent_scope_id;

        self.context.add_scope_binding(node.id, block_scope_id);

        Ok(())
    }

    fn visit_if_expression(
        &mut self,
        node: SyntaxReader,
        _: Self::Input,
    ) -> Result<Self::Output, CompileError> {
        debug!("Binding if expression");

        let children = node
            .multiple_children()
            .ok_or(CompileError::MissingChildren {
                parent_kind: node.inner().kind,
                start_index: node.inner().children.0,
                count: node.inner().children.1,
            })?;

        for child in children {
            self.visit(child, ())?;
        }

        Ok(())
    }

    fn visit_else_expression(
        &mut self,
        node: SyntaxReader,
        _: Self::Input,
    ) -> Result<Self::Output, CompileError> {
        debug!("Binding else expression");

        let child = node.left_child().ok_or(CompileError::MissingChild {
            parent_kind: node.kind(),
            child_index: 0,
        })?;

        self.visit_expression(child, ())?;

        Ok(())
    }

    fn visit_math_binary_expression(
        &mut self,
        node: SyntaxReader,
        _: Self::Input,
    ) -> Result<Self::Output, CompileError> {
        debug!("Binding math binary expression");

        let left_expression = node.left_child().ok_or(CompileError::MissingChild {
            parent_kind: node.kind(),
            child_index: 0,
        })?;
        let right_expression = node.right_child().ok_or(CompileError::MissingChild {
            parent_kind: node.kind(),
            child_index: 1,
        })?;

        self.visit_expression(left_expression, ())?;
        self.visit_expression(right_expression, ())?;

        Ok(())
    }

    fn visit_comparison_binary_expression(
        &mut self,
        node: SyntaxReader,
        _: Self::Input,
    ) -> Result<Self::Output, CompileError> {
        debug!("Binding comparison binary expression");

        let left_expression = node.left_child().ok_or(CompileError::MissingChild {
            parent_kind: node.kind(),
            child_index: 0,
        })?;
        let right_expression = node.right_child().ok_or(CompileError::MissingChild {
            parent_kind: node.kind(),
            child_index: 1,
        })?;

        self.visit_expression(left_expression, ())?;
        self.visit_expression(right_expression, ())?;

        Ok(())
    }

    fn visit_logical_binary_expression(
        &mut self,
        node: SyntaxReader,
        _: Self::Input,
    ) -> Result<Self::Output, CompileError> {
        debug!("Binding logical binary expression");

        let left_expression = node.left_child().ok_or(CompileError::MissingChild {
            parent_kind: node.kind(),
            child_index: 0,
        })?;
        let right_expression = node.right_child().ok_or(CompileError::MissingChild {
            parent_kind: node.kind(),
            child_index: 1,
        })?;

        self.visit_expression(left_expression, ())?;
        self.visit_expression(right_expression, ())?;

        Ok(())
    }

    fn visit_unary_negation_expression(
        &mut self,
        node: SyntaxReader,
        _: Self::Input,
    ) -> Result<Self::Output, CompileError> {
        debug!("Binding unary negation expression");

        let expression = node.left_child().ok_or(CompileError::MissingChild {
            parent_kind: node.kind(),
            child_index: 0,
        })?;

        self.visit_expression(expression, ())?;

        Ok(())
    }

    fn visit_while_expression(
        &mut self,
        node: SyntaxReader,
        _: Self::Input,
    ) -> Result<Self::Output, CompileError> {
        debug!("Binding while expression");

        let condition = node.left_child().ok_or(CompileError::MissingChild {
            parent_kind: node.kind(),
            child_index: 0,
        })?;
        let body = node.right_child().ok_or(CompileError::MissingChild {
            parent_kind: node.kind(),
            child_index: 1,
        })?;

        self.visit_expression(condition, ())?;
        self.visit(body, ())?;

        Ok(())
    }

    fn visit_function_expression(
        &mut self,
        node: SyntaxReader,
        _: Self::Input,
    ) -> Result<Self::Output, CompileError> {
        debug!("Binding function expression");

        let signature = node.left_child().ok_or(CompileError::MissingChild {
            parent_kind: node.kind(),
            child_index: 0,
        })?;
        let value_parameters = signature
            .left_child()
            .ok_or(CompileError::MissingChild {
                parent_kind: signature.kind(),
                child_index: 0,
            })?
            .multiple_children()
            .ok_or(CompileError::MissingChildren {
                parent_kind: signature.kind(),
                start_index: signature.inner().children.0,
                count: signature.inner().children.1,
            })?;
        let body = node.right_child().ok_or(CompileError::MissingChild {
            parent_kind: node.kind(),
            child_index: 1,
        })?;

        let function_scope_id = self.context.add_scope(Scope {
            kind: ScopeKind::Function,
            parent: self.current_scope_id,
            imports: SmallVec::new(),
            modules: SmallVec::new(),
        });

        for value_parameter in value_parameters {
            let parameter_name =
                value_parameter
                    .left_child()
                    .ok_or(CompileError::MissingChild {
                        parent_kind: value_parameter.kind(),
                        child_index: 0,
                    })?;

            let source_file = self.source.files().get(self.file_id.0 as usize).ok_or(
                CompileError::MissingSourceFile {
                    file_id: self.file_id,
                },
            )?;
            let parameter_name_str = source_file.source_code.get_span(parameter_name.span());

            let parameter_declaration = Declaration {
                kind: DeclarationKind::Local { shadowed: None },
                scope_id: function_scope_id,
                position: Position::new(self.file_id, parameter_name.span()),
                is_public: false,
            };
            let parameter_declaration_id = self
                .context
                .add_declaration(parameter_name_str, parameter_declaration);

            self.context
                .set_declaration_binding(parameter_name.id, parameter_declaration_id);
        }

        self.context.add_scope_binding(body.id, function_scope_id);

        let mut function_declaration_binder = DeclarationBinder::new(
            None,
            self.file_id,
            self.source,
            self.syntax,
            self.context,
            function_scope_id,
        );

        function_declaration_binder.visit(body, ())?;

        Ok(())
    }

    fn visit_call_expression(
        &mut self,
        node: SyntaxReader,
        _: Self::Input,
    ) -> Result<Self::Output, CompileError> {
        debug!("Binding call expression");

        let callee = node.left_child().ok_or(CompileError::MissingChild {
            parent_kind: node.kind(),
            child_index: 0,
        })?;
        let arguments = node.right_child().ok_or(CompileError::MissingChild {
            parent_kind: node.kind(),
            child_index: 1,
        })?;

        self.visit_expression(callee, ())?;

        if let Some(argument_nodes) = arguments.multiple_children() {
            for argument in argument_nodes {
                self.visit_expression(argument, ())?;
            }
        }

        Ok(())
    }
}
