use smallvec::SmallVec;
use tracing::{debug, info};

use crate::{
    compiler::{
        CompileError,
        error::InternalError,
        resolver::{
            Declaration, DeclarationId, DeclarationKind, Resolver, Scope, ScopeId, ScopeKind,
            Symbol,
        },
    },
    source::{Source, SourceFileId},
    syntax::{Syntax, SyntaxId, SyntaxKind, SyntaxReader, SyntaxVisitor},
};

pub struct DeclarationBinder<'a> {
    current_scope_id: ScopeId,

    source: &'a Source,

    syntax: &'a Syntax,

    resolver: &'a mut Resolver,
}

impl<'a> DeclarationBinder<'a> {
    pub fn new(
        current_scope_id: ScopeId,
        source: &'a Source,
        syntax: &'a Syntax,
        resolver: &'a mut Resolver,
    ) -> Self {
        Self {
            current_scope_id,
            source,
            syntax,
            resolver,
        }
    }

    pub fn bind_main(mut self) -> Result<DeclarationId, CompileError> {
        let main_root = self
            .syntax
            .get_tree(SourceFileId::MAIN)
            .ok_or(CompileError::Internal(InternalError::MissingSyntaxTree(
                SourceFileId::MAIN,
            )))?
            .root()
            .ok_or(CompileError::Internal(InternalError::MissingSyntaxNode(
                SyntaxId::ROOT,
            )))?;

        let main_scope = self.resolver.add_scope(Scope {
            kind: ScopeKind::Function,
            parent: ScopeId::PROJECT,
            imports: SmallVec::new(),
            modules: SmallVec::new(),
        });
        let main_declaration = Declaration {
            symbol: self.resolver.create_anonymous_symbol(),
            kind: DeclarationKind::Type { parent: None },
            scope_id: main_scope,
            is_public: true,
            position: Some(main_root.position()),
        };
        let main_declaration_id = self.resolver.add_declaration(main_declaration);

        self.visit_main_function_item(main_root, ());

        Ok(main_declaration_id)
    }

    fn create_symbol(&mut self, path: &SyntaxReader) -> Result<Symbol, CompileError> {
        debug_assert!(matches!(
            path.kind(),
            SyntaxKind::Path | SyntaxKind::PathSegment
        ));

        let position = path.position();
        let bytes = self.source.get_source_bytes(&position);
        let constant_id = self.resolver.constants.add_string(bytes);

        Ok(Symbol::Source {
            constant_id,
            position,
        })
    }
}

impl<'a> SyntaxVisitor for DeclarationBinder<'a> {
    type Input = ();

    type Output = ();

    fn visit_main_function_item(
        &mut self,
        node: SyntaxReader,
        _: Self::Input,
    ) -> Result<Self::Output, CompileError> {
        debug!("Binding main function");

        let children = node.multiple_children().ok_or(CompileError::Internal(
            InternalError::MissingSyntaxChildren {
                start_index: node.inner().children.0,
                count: node.inner().children.1,
            },
        ))?;

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

        let function_name =
            node.left_child()
                .ok_or(CompileError::Internal(InternalError::MissingSyntaxChild {
                    child_index: 0,
                }))?;
        let function_expression = node.right_child().ok_or(CompileError::Internal(
            InternalError::MissingSyntaxChild { child_index: 1 },
        ))?;
        let signature = function_expression
            .left_child()
            .ok_or(CompileError::Internal(InternalError::MissingSyntaxChild {
                child_index: 0,
            }))?;
        let value_parameters = signature
            .left_child()
            .ok_or(CompileError::Internal(InternalError::MissingSyntaxChild {
                child_index: 0,
            }))?
            .multiple_children()
            .ok_or(CompileError::Internal(
                InternalError::MissingSyntaxChildren {
                    start_index: signature.inner().children.0,
                    count: signature.inner().children.1,
                },
            ))?;
        let return_type = signature.right_child();
        let function_body = function_expression
            .right_child()
            .ok_or(CompileError::Internal(InternalError::MissingSyntaxChild {
                child_index: 1,
            }))?;

        let function_scope_id = self.resolver.add_scope(Scope {
            kind: ScopeKind::Function,
            parent: self.current_scope_id,
            imports: SmallVec::new(),
            modules: SmallVec::new(),
        });

        let function_symbol = self.create_symbol(&function_name)?;
        let is_public = match node.kind() {
            SyntaxKind::PublicFunctionItem => true,
            SyntaxKind::FunctionItem => false,
            _ => unreachable!(),
        };
        let function_declaration = Declaration {
            symbol: function_symbol,
            kind: DeclarationKind::Type { parent: None },
            scope_id: self.current_scope_id,
            is_public,
            position: Some(signature.position()),
        };
        let function_declaration_id = self.resolver.add_declaration(function_declaration);

        for value_parameter in value_parameters {
            let parameter_name = value_parameter.left_child().ok_or(CompileError::Internal(
                InternalError::MissingSyntaxChild { child_index: 0 },
            ))?;
            let parameter_symbol = self.create_symbol(&parameter_name)?;
            let parameter_declaration = Declaration {
                symbol: parameter_symbol,
                kind: DeclarationKind::Local {
                    shadowed: None,
                    is_mutable: false,
                },
                scope_id: function_scope_id,
                is_public: false,
                position: Some(value_parameter.position()),
            };
            let parameter_declaration_id = self.resolver.add_declaration(parameter_declaration);

            self.resolver
                .set_declaration_binding(parameter_name.id, parameter_declaration_id);
        }

        if let Some(return_type_node) = return_type {
            self.visit_type(return_type_node, ())?;
        }

        self.resolver
            .add_scope_binding(function_body.id, function_scope_id);
        self.resolver
            .set_declaration_binding(function_expression.id, function_declaration_id);

        let mut function_declaration_binder =
            DeclarationBinder::new(function_scope_id, self.source, self.syntax, self.resolver);

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

        let struct_name =
            node.left_child()
                .ok_or(CompileError::Internal(InternalError::MissingSyntaxChild {
                    child_index: 0,
                }))?;
        let struct_fields = node
            .right_child()
            .ok_or(CompileError::Internal(InternalError::MissingSyntaxChild {
                child_index: 1,
            }))?
            .multiple_children()
            .ok_or(CompileError::Internal(
                InternalError::MissingSyntaxChildren {
                    start_index: node.inner().children.0,
                    count: node.inner().children.1,
                },
            ))?;

        let struct_symbol = self.create_symbol(&struct_name)?;
        let struct_declaration = Declaration {
            symbol: struct_symbol,
            kind: DeclarationKind::Type { parent: None },
            scope_id: self.current_scope_id,
            is_public: false,
            position: Some(node.position()),
        };
        let struct_declaration_id = self.resolver.add_declaration(struct_declaration);

        let mut field_ids = SmallVec::<[DeclarationId; 8]>::new();

        for field in struct_fields {
            let field_name = field.left_child().ok_or(CompileError::Internal(
                InternalError::MissingSyntaxChild { child_index: 0 },
            ))?;
            let field_type = field.right_child().ok_or(CompileError::Internal(
                InternalError::MissingSyntaxChild { child_index: 1 },
            ))?;

            let field_symbol = self.create_symbol(&field_name)?;
            let field_declaration = Declaration {
                symbol: field_symbol,
                kind: DeclarationKind::Type {
                    parent: Some(struct_declaration_id),
                },
                scope_id: self.current_scope_id,
                is_public: false,
                position: Some(field.position()),
            };
            let field_declaration_id = self.resolver.add_declaration(field_declaration);

            self.visit_type(field_type, ())?;
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

        let expression =
            node.left_child()
                .ok_or(CompileError::Internal(InternalError::MissingSyntaxChild {
                    child_index: 0,
                }))?;

        self.visit_expression(expression, ())?;

        Ok(())
    }

    fn visit_let_statement(
        &mut self,
        node: SyntaxReader,
        _: Self::Input,
    ) -> Result<Self::Output, CompileError> {
        info!("Binding let statement");

        let mut children = node.multiple_children().ok_or(CompileError::Internal(
            InternalError::MissingSyntaxChildren {
                start_index: node.inner().children.0,
                count: node.inner().children.1,
            },
        ))?;
        let path =
            children
                .next()
                .ok_or(CompileError::Internal(InternalError::MissingSyntaxChild {
                    child_index: 0,
                }))?;
        let expression_statement =
            children
                .next()
                .ok_or(CompileError::Internal(InternalError::MissingSyntaxChild {
                    child_index: 1,
                }))?;
        let expression = expression_statement
            .left_child()
            .ok_or(CompileError::Internal(InternalError::MissingSyntaxChild {
                child_index: 0,
            }))?;

        self.visit_expression(expression, ())?;

        let symbol = self.create_symbol(&path)?;
        let shadowed = self
            .resolver
            .find_declaration_in_scope(symbol, self.current_scope_id, None)
            .map(|(id, _)| id);
        let is_mutable = node.kind() == SyntaxKind::LetMutStatement;
        let declaration = Declaration {
            symbol,
            kind: DeclarationKind::Local {
                shadowed,
                is_mutable,
            },
            scope_id: self.current_scope_id,
            is_public: false,
            position: Some(node.position()),
        };
        let declaration_id = self.resolver.add_declaration(declaration);

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

        let path =
            node.left_child()
                .ok_or(CompileError::Internal(InternalError::MissingSyntaxChild {
                    child_index: 0,
                }))?;
        let expression = node.right_child().ok_or(CompileError::Internal(
            InternalError::MissingSyntaxChild { child_index: 1 },
        ))?;

        self.visit_expression(expression, input)?;

        let symbol = self.create_symbol(&path)?;
        let (declaration_id, _) = self
            .resolver
            .find_declaration_in_scope(symbol, self.current_scope_id, None)
            .ok_or(CompileError::UndeclaredVariable {
                position: path.position(),
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
        let path =
            node.left_child()
                .ok_or(CompileError::Internal(InternalError::MissingSyntaxChild {
                    child_index: 0,
                }))?;
        let expression_statement = node.right_child().ok_or(CompileError::Internal(
            InternalError::MissingSyntaxChild { child_index: 1 },
        ))?;
        let expression = expression_statement
            .left_child()
            .ok_or(CompileError::Internal(InternalError::MissingSyntaxChild {
                child_index: 0,
            }))?;

        let symbol = self.create_symbol(&path)?;
        let (declaration_id, _) = self
            .resolver
            .find_declaration_in_scope(symbol, self.current_scope_id, None)
            .ok_or(CompileError::UndeclaredVariable {
                position: path.position(),
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

        let elements = node.multiple_children().ok_or(CompileError::Internal(
            InternalError::MissingSyntaxChildren {
                start_index: node.inner().children.0,
                count: node.inner().children.1,
            },
        ))?;

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

        let list =
            node.left_child()
                .ok_or(CompileError::Internal(InternalError::MissingSyntaxChild {
                    child_index: 0,
                }))?;
        let index = node.right_child().ok_or(CompileError::Internal(
            InternalError::MissingSyntaxChild { child_index: 1 },
        ))?;

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

        let path =
            node.left_child()
                .ok_or(CompileError::Internal(InternalError::MissingSyntaxChild {
                    child_index: 0,
                }))?;
        let path_segments = path.multiple_children().ok_or(CompileError::Internal(
            InternalError::MissingSyntaxChildren {
                start_index: path.inner().children.0,
                count: path.inner().children.1,
            },
        ))?;

        let mut current_declaration_id = DeclarationId(0);
        let mut current_scope_id = self.current_scope_id;

        for segment in path_segments {
            let symbol = self.create_symbol(&segment)?;
            let (next_declaration_id, next_declaration) = self
                .resolver
                .find_declaration_in_scope(symbol, current_scope_id, None)
                .ok_or(CompileError::UndeclaredVariable {
                    position: segment.position(),
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

        let path =
            node.left_child()
                .ok_or(CompileError::Internal(InternalError::MissingSyntaxChild {
                    child_index: 0,
                }))?;
        let fields = node
            .right_child()
            .ok_or(CompileError::Internal(InternalError::MissingSyntaxChild {
                child_index: 1,
            }))?
            .multiple_children()
            .ok_or(CompileError::Internal(
                InternalError::MissingSyntaxChildren {
                    start_index: node.inner().children.0,
                    count: node.inner().children.1,
                },
            ))?;

        let struct_symbol = self.create_symbol(&path)?;

        let (struct_declaration_id, struct_declaration) = self
            .resolver
            .find_declaration_in_scope(struct_symbol, self.current_scope_id, None)
            .ok_or(CompileError::UndeclaredVariable {
                position: path.position(),
            })?;

        self.resolver
            .set_declaration_binding(path.id, struct_declaration_id);
        self.resolver
            .set_declaration_binding(node.id, struct_declaration_id);

        for field in fields {
            let field_path = field.left_child().ok_or(CompileError::Internal(
                InternalError::MissingSyntaxChild { child_index: 0 },
            ))?;
            let field_value = field.right_child().ok_or(CompileError::Internal(
                InternalError::MissingSyntaxChild { child_index: 1 },
            ))?;

            let field_symbol = self.create_symbol(&field_path)?;
            let (field_declaration_id, _) = self
                .resolver
                .find_declaration_in_scope(
                    field_symbol,
                    struct_declaration.scope_id,
                    Some(struct_declaration_id),
                )
                .ok_or(CompileError::UndeclaredVariable {
                    position: field_path.position(),
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

        let children = node.multiple_children().ok_or(CompileError::Internal(
            InternalError::MissingSyntaxChildren {
                start_index: node.inner().children.0,
                count: node.inner().children.1,
            },
        ))?;

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

        let children = node.multiple_children().ok_or(CompileError::Internal(
            InternalError::MissingSyntaxChildren {
                start_index: node.inner().children.0,
                count: node.inner().children.1,
            },
        ))?;

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

        let child =
            node.left_child()
                .ok_or(CompileError::Internal(InternalError::MissingSyntaxChild {
                    child_index: 0,
                }))?;

        self.visit_expression(child, ())?;

        Ok(())
    }

    fn visit_math_binary_expression(
        &mut self,
        node: SyntaxReader,
        _: Self::Input,
    ) -> Result<Self::Output, CompileError> {
        debug!("Binding math binary expression");

        let left_expression =
            node.left_child()
                .ok_or(CompileError::Internal(InternalError::MissingSyntaxChild {
                    child_index: 0,
                }))?;
        let right_expression = node.right_child().ok_or(CompileError::Internal(
            InternalError::MissingSyntaxChild { child_index: 1 },
        ))?;

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

        let left_expression =
            node.left_child()
                .ok_or(CompileError::Internal(InternalError::MissingSyntaxChild {
                    child_index: 0,
                }))?;
        let right_expression = node.right_child().ok_or(CompileError::Internal(
            InternalError::MissingSyntaxChild { child_index: 1 },
        ))?;

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

        let left_expression =
            node.left_child()
                .ok_or(CompileError::Internal(InternalError::MissingSyntaxChild {
                    child_index: 0,
                }))?;
        let right_expression = node.right_child().ok_or(CompileError::Internal(
            InternalError::MissingSyntaxChild { child_index: 1 },
        ))?;

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

        let expression =
            node.left_child()
                .ok_or(CompileError::Internal(InternalError::MissingSyntaxChild {
                    child_index: 0,
                }))?;

        self.visit_expression(expression, ())?;

        Ok(())
    }

    fn visit_while_expression(
        &mut self,
        node: SyntaxReader,
        _: Self::Input,
    ) -> Result<Self::Output, CompileError> {
        debug!("Binding while expression");

        let condition =
            node.left_child()
                .ok_or(CompileError::Internal(InternalError::MissingSyntaxChild {
                    child_index: 0,
                }))?;
        let body = node.right_child().ok_or(CompileError::Internal(
            InternalError::MissingSyntaxChild { child_index: 1 },
        ))?;

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

        let signature =
            node.left_child()
                .ok_or(CompileError::Internal(InternalError::MissingSyntaxChild {
                    child_index: 0,
                }))?;
        let value_parameters = signature
            .left_child()
            .ok_or(CompileError::Internal(InternalError::MissingSyntaxChild {
                child_index: 0,
            }))?
            .multiple_children()
            .ok_or(CompileError::Internal(
                InternalError::MissingSyntaxChildren {
                    start_index: signature.inner().children.0,
                    count: signature.inner().children.1,
                },
            ))?;
        let body = node.right_child().ok_or(CompileError::Internal(
            InternalError::MissingSyntaxChild { child_index: 1 },
        ))?;

        let function_scope_id = self.resolver.add_scope(Scope {
            kind: ScopeKind::Function,
            parent: self.current_scope_id,
            imports: SmallVec::new(),
            modules: SmallVec::new(),
        });

        let mut parameter_ids = SmallVec::<[DeclarationId; 8]>::new();

        for value_parameter in value_parameters {
            let parameter_name = value_parameter.left_child().ok_or(CompileError::Internal(
                InternalError::MissingSyntaxChild { child_index: 0 },
            ))?;

            let parameter_symbol = self.create_symbol(&parameter_name)?;
            let parameter_declaration = Declaration {
                symbol: parameter_symbol,
                kind: DeclarationKind::Local {
                    shadowed: None,
                    is_mutable: false,
                },
                scope_id: function_scope_id,
                is_public: false,
                position: Some(value_parameter.position()),
            };
            let parameter_declaration_id = self.resolver.add_declaration(parameter_declaration);

            self.resolver
                .set_declaration_binding(parameter_name.id, parameter_declaration_id);
            parameter_ids.push(parameter_declaration_id);
        }

        let function_symbol = self.resolver.create_anonymous_symbol();
        let function_declaration = Declaration {
            symbol: function_symbol,
            kind: DeclarationKind::Type { parent: None },
            scope_id: self.current_scope_id,
            is_public: false,
            position: Some(signature.position()),
        };
        let function_declaration_id = self.resolver.add_declaration(function_declaration);

        self.resolver.add_scope_binding(body.id, function_scope_id);
        self.resolver
            .set_declaration_binding(node.id, function_declaration_id);

        let starting_scope_id = self.current_scope_id;
        self.current_scope_id = function_scope_id;

        self.visit(body, ())?;

        self.current_scope_id = starting_scope_id;

        Ok(())
    }

    fn visit_call_expression(
        &mut self,
        node: SyntaxReader,
        _: Self::Input,
    ) -> Result<Self::Output, CompileError> {
        debug!("Binding call expression");

        let callee =
            node.left_child()
                .ok_or(CompileError::Internal(InternalError::MissingSyntaxChild {
                    child_index: 0,
                }))?;
        let arguments = node.right_child().ok_or(CompileError::Internal(
            InternalError::MissingSyntaxChild { child_index: 1 },
        ))?;

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

            let path = node.left_child().ok_or(CompileError::Internal(
                InternalError::MissingSyntaxChild { child_index: 0 },
            ))?;
            let path_segments = path.multiple_children().ok_or(CompileError::Internal(
                InternalError::MissingSyntaxChildren {
                    start_index: path.inner().children.0,
                    count: path.inner().children.1,
                },
            ))?;

            let mut current_declaration_id = None;

            for segment in path_segments {
                let segment_symbol = self.create_symbol(&segment)?;
                let declaration_id = if let Some((id, _)) = self.resolver.find_declaration_in_scope(
                    segment_symbol,
                    self.current_scope_id,
                    current_declaration_id,
                ) {
                    id
                } else {
                    let declaration = Declaration {
                        symbol: segment_symbol,
                        kind: DeclarationKind::Type {
                            parent: current_declaration_id,
                        },
                        scope_id: self.current_scope_id,
                        is_public: false,
                        position: None,
                    };

                    self.resolver.add_declaration(declaration)
                };

                current_declaration_id = Some(declaration_id);
            }

            let declaration_id =
                current_declaration_id.ok_or(CompileError::UndeclaredVariable {
                    position: path.position(),
                })?;

            self.resolver
                .set_declaration_binding(node.id, declaration_id);
        }

        Ok(())
    }
}
