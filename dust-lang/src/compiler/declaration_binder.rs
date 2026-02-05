use smallvec::SmallVec;
use tracing::{debug, info};

use crate::{
    compiler::{
        CompileError,
        resolver::{
            Declaration, DeclarationId, DeclarationKind, Resolver, Scope, ScopeId, ScopeKind,
        },
    },
    source::{Position, Source, SourceFileId},
    syntax::{Syntax, SyntaxId, SyntaxKind, SyntaxReader, SyntaxVisitor},
};

pub struct DeclarationBinder<'a> {
    file_id: SourceFileId,

    current_scope_id: ScopeId,

    source: &'a Source,

    syntax: &'a Syntax,

    resolver: &'a mut Resolver,
}

impl<'a> DeclarationBinder<'a> {
    pub fn new(
        file_id: SourceFileId,
        current_scope_id: ScopeId,
        source: &'a Source,
        syntax: &'a Syntax,
        resolver: &'a mut Resolver,
    ) -> Self {
        Self {
            file_id,
            current_scope_id,
            source,
            syntax,
            resolver,
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

        self.resolver.add_scope(Scope {
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
        let return_type = signature.right_child();
        let function_body =
            function_expression
                .right_child()
                .ok_or(CompileError::MissingChild {
                    parent_kind: function_expression.kind(),
                    child_index: 1,
                })?;

        let function_scope_id = self.resolver.add_scope(Scope {
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
                kind: DeclarationKind::Local {
                    shadowed: None,
                    is_mutable: false,
                },
                scope_id: function_scope_id,
                name_position: Some(Position::new(self.file_id, parameter_name.span())),
                is_public: false,
            };
            let parameter_declaration_id = self
                .resolver
                .add_named_declaration(parameter_name_str, parameter_declaration);

            self.resolver
                .set_declaration_binding(parameter_name.id, parameter_declaration_id);
            parameter_ids.push(parameter_declaration_id);
        }

        let is_public = match node.kind() {
            SyntaxKind::PublicFunctionItem => true,
            SyntaxKind::FunctionItem => false,
            _ => unreachable!(),
        };
        let parameters = self.resolver.add_declaration_members(&parameter_ids);
        let function_declaration = Declaration {
            kind: DeclarationKind::Function {
                parameters,
                prototype_index: None,
            },
            scope_id: self.current_scope_id,
            name_position: Some(Position::new(self.file_id, function_name.span())),
            is_public,
        };

        let source_file = self.source.files().get(self.file_id.0 as usize).ok_or(
            CompileError::MissingSourceFile {
                file_id: self.file_id,
            },
        )?;
        let function_name_str = source_file.source_code.get_span(function_name.span());
        let function_declaration_id = self
            .resolver
            .add_named_declaration(function_name_str, function_declaration);

        if let Some(return_type_node) = return_type {
            self.visit_type(return_type_node, ())?;
        }

        self.resolver
            .add_scope_binding(function_body.id, function_scope_id);
        self.resolver
            .set_declaration_binding(function_expression.id, function_declaration_id);

        let mut function_declaration_binder = DeclarationBinder::new(
            self.file_id,
            function_scope_id,
            self.source,
            self.syntax,
            self.resolver,
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

    fn visit_struct_item(
        &mut self,
        node: SyntaxReader,
        _: Self::Input,
    ) -> Result<Self::Output, CompileError> {
        debug!("Binding struct item");

        let struct_name = node.left_child().ok_or(CompileError::MissingChild {
            parent_kind: node.kind(),
            child_index: 0,
        })?;
        let struct_fields = node
            .right_child()
            .ok_or(CompileError::MissingChild {
                parent_kind: node.kind(),
                child_index: 1,
            })?
            .multiple_children()
            .ok_or(CompileError::MissingChildren {
                parent_kind: node.kind(),
                start_index: node.inner().children.0,
                count: node.inner().children.1,
            })?;

        let source_file = self.source.files().get(self.file_id.0 as usize).ok_or(
            CompileError::MissingSourceFile {
                file_id: self.file_id,
            },
        )?;

        let struct_name_str = source_file.source_code.get_span(struct_name.span());
        let struct_declaration = Declaration {
            kind: DeclarationKind::Type { parent: None },
            scope_id: self.current_scope_id,
            name_position: Some(Position::new(self.file_id, struct_name.span())),
            is_public: false,
        };
        let struct_declaration_id = self
            .resolver
            .add_named_declaration(struct_name_str, struct_declaration);

        let mut field_ids = SmallVec::<[DeclarationId; 8]>::new();

        for field in struct_fields {
            let field_name = field.left_child().ok_or(CompileError::MissingChild {
                parent_kind: field.kind(),
                child_index: 0,
            })?;
            let field_type = field.right_child().ok_or(CompileError::MissingChild {
                parent_kind: field.kind(),
                child_index: 1,
            })?;

            let field_name_str = source_file.source_code.get_span(field_name.span());
            let field_declaration = Declaration {
                kind: DeclarationKind::Type {
                    parent: Some(struct_declaration_id),
                },
                scope_id: self.current_scope_id,
                name_position: Some(Position::new(self.file_id, field_name.span())),
                is_public: false,
            };
            let field_declaration_id = self
                .resolver
                .add_named_declaration(field_name_str, field_declaration);

            self.resolver
                .set_declaration_binding(field_name.id, field_declaration_id);
            self.resolver
                .add_scope_binding(field_type.id, self.current_scope_id);
            field_ids.push(field_declaration_id);
        }

        self.resolver
            .set_declaration_binding(struct_name.id, struct_declaration_id);

        Ok(())
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
            .resolver
            .find_declaration_in_scope(variable_name, self.current_scope_id, None)
            .map(|(id, _)| id);
        let is_mutable = node.kind() == SyntaxKind::LetMutStatement;
        let declaration = Declaration {
            kind: DeclarationKind::Local {
                shadowed,
                is_mutable,
            },
            scope_id: self.current_scope_id,
            name_position: Some(Position::new(self.file_id, path.span())),
            is_public: false,
        };
        let declaration_id = self
            .resolver
            .add_named_declaration(variable_name, declaration);

        self.resolver
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
            .resolver
            .find_declaration_in_scope(variable_name, self.current_scope_id, None)
            .ok_or(CompileError::UndeclaredVariable {
                name: variable_name.to_string(),
                position: Position::new(self.file_id, path.span()),
            })?;

        self.resolver
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
            .resolver
            .find_declaration_in_scope(variable_name, self.current_scope_id, None)
            .ok_or(CompileError::UndeclaredVariable {
                name: variable_name.to_string(),
                position: Position::new(self.file_id, path.span()),
            })?;

        self.resolver
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

        let source_file = self.source.files().get(self.file_id.0 as usize).ok_or(
            CompileError::MissingSourceFile {
                file_id: self.file_id,
            },
        )?;

        let mut current_declaration_id = DeclarationId(0);
        let mut current_scope_id = self.current_scope_id;

        for segment in path_segments {
            let segment_name = source_file.source_code.get_span(segment.span());
            let (next_declaration_id, next_declaration) = self
                .resolver
                .find_declaration_in_scope(segment_name, current_scope_id, None)
                .ok_or(CompileError::UndeclaredVariable {
                    name: segment_name.to_string(),
                    position: Position::new(self.file_id, segment.span()),
                })?;

            current_declaration_id = next_declaration_id;
            current_scope_id = next_declaration.scope_id;
        }

        self.resolver
            .set_declaration_binding(node.id, current_declaration_id);

        Ok(())
    }

    fn visit_struct_expression(
        &mut self,
        node: SyntaxReader,
        _: Self::Input,
    ) -> Result<Self::Output, CompileError> {
        debug!("Binding struct expression");

        let path = node.left_child().ok_or(CompileError::MissingChild {
            parent_kind: node.kind(),
            child_index: 0,
        })?;
        let fields = node
            .right_child()
            .ok_or(CompileError::MissingChild {
                parent_kind: node.kind(),
                child_index: 1,
            })?
            .multiple_children()
            .ok_or(CompileError::MissingChildren {
                parent_kind: node.kind(),
                start_index: node.inner().children.0,
                count: node.inner().children.1,
            })?;

        let source_file = self.source.files().get(self.file_id.0 as usize).ok_or(
            CompileError::MissingSourceFile {
                file_id: self.file_id,
            },
        )?;

        let struct_name = source_file.source_code.get_span(path.span());
        let (struct_declaration_id, struct_declaration) = {
            let found = self.resolver.find_declarations(struct_name);

            match found.len() {
                0 => {
                    return Err(CompileError::UndeclaredVariable {
                        name: struct_name.to_string(),
                        position: Position::new(self.file_id, path.span()),
                    });
                }
                1 => found[0],
                _ => {
                    return Err(CompileError::AmbiguousType {
                        name: struct_name.to_string(),
                        position: Position::new(self.file_id, path.span()),
                    });
                }
            }
        };

        self.resolver
            .set_declaration_binding(path.id, struct_declaration_id);
        self.resolver
            .set_declaration_binding(node.id, struct_declaration_id);

        for field in fields {
            let field_path = field.left_child().ok_or(CompileError::MissingChild {
                parent_kind: field.kind(),
                child_index: 0,
            })?;
            let field_value = field.right_child().ok_or(CompileError::MissingChild {
                parent_kind: field.kind(),
                child_index: 1,
            })?;

            let field_name = source_file.source_code.get_span(field_path.span());
            let (field_declaration_id, _) = self
                .resolver
                .find_declaration_in_scope(
                    field_name,
                    struct_declaration.scope_id,
                    Some(struct_declaration_id),
                )
                .ok_or(CompileError::UndeclaredVariable {
                    name: field_name.to_string(),
                    position: Position::new(self.file_id, field_path.span()),
                })?;

            self.resolver
                .set_declaration_binding(field_path.id, field_declaration_id);
            self.visit_expression(field_value, ())?;
        }

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

        let block_scope_id = self.resolver.add_scope(Scope {
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

        self.resolver.add_scope_binding(node.id, block_scope_id);

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

        let function_scope_id = self.resolver.add_scope(Scope {
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
                kind: DeclarationKind::Local {
                    shadowed: None,
                    is_mutable: false,
                },
                scope_id: function_scope_id,
                name_position: Some(Position::new(self.file_id, parameter_name.span())),
                is_public: false,
            };
            let parameter_declaration_id = self
                .resolver
                .add_named_declaration(parameter_name_str, parameter_declaration);

            self.resolver
                .set_declaration_binding(parameter_name.id, parameter_declaration_id);
            parameter_ids.push(parameter_declaration_id);
        }

        let parameters = self.resolver.add_declaration_members(&parameter_ids);
        let function_declaration = Declaration {
            kind: DeclarationKind::Function {
                parameters,
                prototype_index: None,
            },
            scope_id: self.current_scope_id,
            name_position: None,
            is_public: false,
        };

        let function_declaration_id = self
            .resolver
            .add_anonymous_declaration(function_declaration);

        self.resolver.add_scope_binding(body.id, function_scope_id);
        self.resolver
            .set_declaration_binding(node.id, function_declaration_id);

        let mut function_declaration_binder = DeclarationBinder::new(
            self.file_id,
            function_scope_id,
            self.source,
            self.syntax,
            self.resolver,
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

    fn visit_type(
        &mut self,
        node: SyntaxReader,
        _: Self::Input,
    ) -> Result<Self::Output, CompileError> {
        if node.kind() == SyntaxKind::TypePath {
            debug!("Binding path type");

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

            let file = self.source.files().get(self.file_id.0 as usize).ok_or(
                CompileError::MissingSourceFile {
                    file_id: self.file_id,
                },
            )?;

            let mut current_declaration_id = None;

            for segment in path_segments {
                let segment_name = file.source_code.get_span(segment.span());
                let (declaration_id, _) = self
                    .resolver
                    .find_declaration_in_scope(
                        segment_name,
                        self.current_scope_id,
                        current_declaration_id,
                    )
                    .ok_or(CompileError::UndeclaredType {
                        name: segment_name.to_string(),
                        position: Position::new(self.file_id, node.span()),
                    })?;

                current_declaration_id = Some(declaration_id);
            }

            let declaration_id =
                current_declaration_id.ok_or(CompileError::UndeclaredVariable {
                    name: String::new(),
                    position: Position::new(self.file_id, path.span()),
                })?;

            self.resolver
                .set_declaration_binding(node.id, declaration_id);
        }

        Ok(())
    }
}
