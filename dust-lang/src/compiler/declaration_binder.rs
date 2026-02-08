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

        self.visit_main(main_root)
    }
}

impl SyntaxVisitor for DeclarationBinder<'_> {
    type MainOutput = DeclarationId;
    type ItemOutput = ();
    type StatementOutput = ();
    type ExpressionInput = ();
    type ExpressionOutput = ();
    type TypeOutput = ();
    type PathOutput = DeclarationId;

    fn visit_main(&mut self, node: SyntaxReader) -> Result<Self::MainOutput, CompileError> {
        debug!("Binding main function");

        let children = node.multiple_children()?;

        let main_scope = self.resolver.add_scope(Scope {
            kind: ScopeKind::Function,
            parent: ScopeId::PROJECT,
            imports: SmallVec::new(),
            modules: SmallVec::new(),
        });
        let main_declaration_id = self.resolver.add_declaration(Declaration {
            symbol: Symbol::MAIN,
            kind: DeclarationKind::Type { parent: None },
            scope_id: main_scope,
            is_public: true,
            position: Some(node.position()),
        });

        self.resolver
            .add_declaration_binding(node.id, main_declaration_id);
        self.resolver.add_scope_binding(node.id, main_scope);

        for child in children {
            if child.kind().is_item() {
                self.visit_item(child)?;
            } else if child.kind().is_statement() {
                self.visit_statement(child)?;
            } else {
                self.visit_expression(child, ())?;
            }
        }

        Ok(main_declaration_id)
    }

    fn visit_module_item(&mut self, _: SyntaxReader) -> Result<Self::ItemOutput, CompileError> {
        debug!("Binding module item");

        todo!()
    }

    fn visit_function_item(
        &mut self,
        node: SyntaxReader,
    ) -> Result<Self::ItemOutput, CompileError> {
        debug!("Binding function item");

        let (function_name, function_expression) = node.binary_children()?;
        let (signature, body) = function_expression.binary_children()?;
        let value_parameter_list = signature.left_child()?;
        let value_parameters = value_parameter_list.multiple_children()?;
        let return_type = if signature.has_right_child() {
            let right = signature.right_child()?;

            Some(right)
        } else {
            None
        };

        let function_scope_id = self.resolver.add_scope(Scope {
            kind: ScopeKind::Function,
            parent: self.current_scope_id,
            imports: SmallVec::new(),
            modules: SmallVec::new(),
        });

        let function_symbol = self.resolver.create_symbol(&function_name, self.source);
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
            let parameter_name = value_parameter.left_child().map_err(CompileError::Syntax)?;
            let parameter_declaration = Declaration {
                symbol: self.resolver.create_symbol(&parameter_name, self.source),
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
                .add_declaration_binding(parameter_name.id, parameter_declaration_id);
        }

        if let Some(type_node) = return_type {
            self.visit_type(type_node)?;
        }

        self.resolver.add_scope_binding(body.id, function_scope_id);
        self.resolver
            .add_declaration_binding(function_expression.id, function_declaration_id);

        let mut function_declaration_binder =
            DeclarationBinder::new(function_scope_id, self.source, self.syntax, self.resolver);

        function_declaration_binder.visit_block_expression(body, ())?;

        Ok(())
    }

    fn visit_use_item(&mut self, _: SyntaxReader) -> Result<Self::ItemOutput, CompileError> {
        debug!("Binding use item");

        todo!()
    }

    fn visit_struct_item(&mut self, node: SyntaxReader) -> Result<Self::ItemOutput, CompileError> {
        debug!("Binding struct item");

        let (struct_name, struct_fields_list) = node.binary_children()?;
        let struct_fields = struct_fields_list.multiple_children()?;

        let struct_symbol = self.resolver.create_symbol(&struct_name, self.source);
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
            let (field_name, field_type) = field.binary_children()?;

            let field_symbol = self.resolver.create_symbol(&field_name, self.source);
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

            self.visit_type(field_type)?;
            self.resolver
                .add_declaration_binding(field_name.id, field_declaration_id);
            self.resolver
                .add_scope_binding(field_type.id, self.current_scope_id);
            field_ids.push(field_declaration_id);
        }

        self.resolver
            .add_declaration_binding(struct_name.id, struct_declaration_id);

        Ok(())
    }

    fn visit_expression_statement(
        &mut self,
        node: SyntaxReader,
    ) -> Result<Self::StatementOutput, CompileError> {
        debug!("Binding expression statement");

        self.visit_expression(node.left_child()?, ())
    }

    fn visit_let_statement(
        &mut self,
        node: SyntaxReader,
    ) -> Result<Self::StatementOutput, CompileError> {
        info!("Binding let statement");

        let mut children = node.multiple_children()?;
        let path = children.expect_next()?;
        let expression_statement = children.expect_next()?;
        let expression = expression_statement.left_child()?;

        self.visit_expression(expression, ())?;

        let symbol = self.resolver.create_symbol(&path, self.source);
        let shadowed = self
            .resolver
            .find_declaration_in_scope(symbol, &path, self.current_scope_id, None, false)
            .map(|(id, _)| id)
            .ok();
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
            .add_declaration_binding(path.id, declaration_id);

        Ok(())
    }

    fn visit_binary_assignment_statement(
        &mut self,
        node: SyntaxReader,
    ) -> Result<Self::StatementOutput, CompileError> {
        info!("Binding binary assignment statement");

        let (path, expression) = node.binary_children()?;

        self.visit_expression(expression, ())?;

        let symbol = self.resolver.create_symbol(&path, self.source);
        let (declaration_id, _) = self.resolver.find_declaration_in_scope(
            symbol,
            &path,
            self.current_scope_id,
            None,
            false,
        )?;

        self.resolver
            .add_declaration_binding(path.id, declaration_id);

        Ok(())
    }

    fn visit_reassignment_statement(
        &mut self,
        node: SyntaxReader,
    ) -> Result<Self::StatementOutput, CompileError> {
        let (path, expression_statement) = node.binary_children()?;
        let expression = expression_statement.left_child()?;

        let symbol = self.resolver.create_symbol(&path, self.source);
        let (declaration_id, _) = self.resolver.find_declaration_in_scope(
            symbol,
            &path,
            self.current_scope_id,
            None,
            false,
        )?;

        self.resolver
            .add_declaration_binding(path.id, declaration_id);
        self.visit_expression(expression, ())?;

        Ok(())
    }

    fn visit_boolean_expression(
        &mut self,
        _: SyntaxReader,
        _: Self::ExpressionInput,
    ) -> Result<Self::ExpressionOutput, CompileError> {
        Ok(())
    }

    fn visit_byte_expression(
        &mut self,
        _: SyntaxReader,
        _: Self::ExpressionInput,
    ) -> Result<Self::ExpressionOutput, CompileError> {
        Ok(())
    }

    fn visit_character_expression(
        &mut self,
        _: SyntaxReader,
        _: Self::ExpressionInput,
    ) -> Result<Self::ExpressionOutput, CompileError> {
        Ok(())
    }

    fn visit_float_expression(
        &mut self,
        _: SyntaxReader,
        _: Self::ExpressionInput,
    ) -> Result<Self::ExpressionOutput, CompileError> {
        Ok(())
    }

    fn visit_integer_expression(
        &mut self,
        _: SyntaxReader,
        _: Self::ExpressionInput,
    ) -> Result<Self::ExpressionOutput, CompileError> {
        Ok(())
    }

    fn visit_string_expression(
        &mut self,
        _: SyntaxReader,
        _: Self::ExpressionInput,
    ) -> Result<Self::ExpressionOutput, CompileError> {
        Ok(())
    }

    fn visit_list_expression(
        &mut self,
        node: SyntaxReader,
        _: Self::ExpressionInput,
    ) -> Result<Self::ExpressionOutput, CompileError> {
        debug!("Binding list expression");

        for element in node.multiple_children()? {
            self.visit_expression(element, ())?;
        }

        Ok(())
    }

    fn visit_index_expression(
        &mut self,
        node: SyntaxReader,
        _: Self::ExpressionInput,
    ) -> Result<Self::ExpressionOutput, CompileError> {
        debug!("Binding index expression");

        let (list, index) = node.binary_children()?;

        self.visit_expression(list, ())?;
        self.visit_expression(index, ())?;

        Ok(())
    }

    fn visit_path_expression(
        &mut self,
        _: SyntaxReader,
        _: Self::ExpressionInput,
    ) -> Result<Self::ExpressionOutput, CompileError> {
        Ok(())
    }

    fn visit_struct_expression(
        &mut self,
        node: SyntaxReader,
        _: Self::ExpressionInput,
    ) -> Result<Self::ExpressionOutput, CompileError> {
        debug!("Binding struct expression");

        let (path, fields_list) = node.binary_children()?;
        let fields = fields_list.multiple_children()?;

        let struct_symbol = self.resolver.create_symbol(&path, self.source);
        let (struct_declaration_id, struct_declaration) = self.resolver.find_declaration_in_scope(
            struct_symbol,
            &path,
            self.current_scope_id,
            None,
            true,
        )?;

        self.resolver
            .add_declaration_binding(path.id, struct_declaration_id);
        self.resolver
            .add_declaration_binding(node.id, struct_declaration_id);

        for field in fields {
            let (field_path, field_value) = field.binary_children()?;

            let field_symbol = self.resolver.create_symbol(&field_path, self.source);
            let (field_declaration_id, _) = self.resolver.find_declaration_in_scope(
                field_symbol,
                &path,
                struct_declaration.scope_id,
                Some(struct_declaration_id),
                true,
            )?;

            self.resolver
                .add_declaration_binding(field_path.id, field_declaration_id);
            self.visit_expression(field_value, ())?;
        }

        Ok(())
    }

    fn visit_block_expression(
        &mut self,
        node: SyntaxReader,
        _: Self::ExpressionInput,
    ) -> Result<Self::ExpressionOutput, CompileError> {
        debug!("Binding block expression");

        let children = node.multiple_children()?;

        let block_scope_id = self.resolver.add_scope(Scope {
            kind: ScopeKind::Block,
            parent: self.current_scope_id,
            imports: SmallVec::new(),
            modules: SmallVec::new(),
        });
        let parent_scope_id = self.current_scope_id;
        self.current_scope_id = block_scope_id;

        for child in children {
            if child.kind().is_item() {
                self.visit_item(child)?;
            } else if child.kind().is_statement() {
                self.visit_statement(child)?;
            } else {
                self.visit_expression(child, ())?;
            }
        }

        self.current_scope_id = parent_scope_id;

        self.resolver.add_scope_binding(node.id, block_scope_id);

        Ok(())
    }

    fn visit_if_expression(
        &mut self,
        node: SyntaxReader,
        _: Self::ExpressionInput,
    ) -> Result<Self::ExpressionOutput, CompileError> {
        debug!("Binding if expression");

        let mut children = node.multiple_children()?;
        let condition = children.expect_next()?;
        let then_branch = children.expect_next()?;
        let else_branch = children.next();

        self.visit_expression(condition, ())?;
        self.visit_block_expression(then_branch, ())?;

        if let Some(else_branch) = else_branch {
            self.visit_else_expression(else_branch, ())?;
        }

        Ok(())
    }

    fn visit_else_expression(
        &mut self,
        node: SyntaxReader,
        _: Self::ExpressionInput,
    ) -> Result<Self::ExpressionOutput, CompileError> {
        debug!("Binding else expression");

        self.visit_expression(node.left_child()?, ())?;

        Ok(())
    }

    fn visit_math_binary_expression(
        &mut self,
        node: SyntaxReader,
        _: Self::ExpressionInput,
    ) -> Result<Self::ExpressionOutput, CompileError> {
        debug!("Binding math binary expression");

        let (left_expression, right_expression) = node.binary_children()?;

        self.visit_expression(left_expression, ())?;
        self.visit_expression(right_expression, ())?;

        Ok(())
    }

    fn visit_comparison_binary_expression(
        &mut self,
        node: SyntaxReader,
        _: Self::ExpressionInput,
    ) -> Result<Self::ExpressionOutput, CompileError> {
        debug!("Binding comparison binary expression");

        let (left_expression, right_expression) = node.binary_children()?;

        self.visit_expression(left_expression, ())?;
        self.visit_expression(right_expression, ())?;

        Ok(())
    }

    fn visit_logical_binary_expression(
        &mut self,
        node: SyntaxReader,
        _: Self::ExpressionInput,
    ) -> Result<Self::ExpressionOutput, CompileError> {
        debug!("Binding logical binary expression");

        let (left_expression, right_expression) = node.binary_children()?;

        self.visit_expression(left_expression, ())?;
        self.visit_expression(right_expression, ())?;

        Ok(())
    }

    fn visit_unary_negation_expression(
        &mut self,
        node: SyntaxReader,
        _: Self::ExpressionInput,
    ) -> Result<Self::ExpressionOutput, CompileError> {
        debug!("Binding unary negation expression");

        self.visit_expression(node.left_child()?, ())?;

        Ok(())
    }

    fn visit_while_expression(
        &mut self,
        node: SyntaxReader,
        _: Self::ExpressionInput,
    ) -> Result<Self::ExpressionOutput, CompileError> {
        debug!("Binding while expression");

        let (condition, body) = node.binary_children()?;

        self.visit_expression(condition, ())?;
        self.visit_block_expression(body, ())?;

        Ok(())
    }

    fn visit_function_expression(
        &mut self,
        node: SyntaxReader,
        _: Self::ExpressionInput,
    ) -> Result<Self::ExpressionOutput, CompileError> {
        debug!("Binding function expression");

        let (signature, body) = node.binary_children()?;
        let value_parameters = signature.left_child()?.multiple_children()?;

        let function_scope_id = self.resolver.add_scope(Scope {
            kind: ScopeKind::Function,
            parent: self.current_scope_id,
            imports: SmallVec::new(),
            modules: SmallVec::new(),
        });

        let mut parameter_ids = SmallVec::<[DeclarationId; 8]>::new();

        for value_parameter in value_parameters {
            let parameter_name = value_parameter.left_child()?;

            let parameter_symbol = self.resolver.create_symbol(&parameter_name, self.source);
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
                .add_declaration_binding(parameter_name.id, parameter_declaration_id);
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
            .add_declaration_binding(node.id, function_declaration_id);

        let starting_scope_id = self.current_scope_id;
        self.current_scope_id = function_scope_id;

        self.visit_block_expression(body, ())?;

        self.current_scope_id = starting_scope_id;

        Ok(())
    }

    fn visit_call_expression(
        &mut self,
        node: SyntaxReader,
        _: Self::ExpressionInput,
    ) -> Result<Self::ExpressionOutput, CompileError> {
        debug!("Binding call expression");

        let (callee, arguments_list) = node.binary_children()?;
        let arguments = arguments_list.multiple_children()?;

        self.visit_expression(callee, ())?;

        for argument in arguments {
            self.visit_expression(argument, ())?;
        }

        Ok(())
    }

    fn visit_type(&mut self, node: SyntaxReader) -> Result<Self::TypeOutput, CompileError> {
        if node.kind() == SyntaxKind::TypePath {
            debug!("Binding path type");

            let path = node.left_child()?;
            let path_segments = path.multiple_children()?;

            let mut current_declaration_id = None;

            for segment in path_segments {
                let segment_symbol = self.resolver.create_symbol(&segment, self.source);
                let declaration_id = if let Ok((id, _)) = self.resolver.find_declaration_in_scope(
                    segment_symbol,
                    &segment,
                    self.current_scope_id,
                    current_declaration_id,
                    true,
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

            let declaration_id = current_declaration_id.ok_or(CompileError::UndeclaredType {
                name: self.resolver.create_symbol(&path, self.source),
                position: node.position(),
            })?;

            self.resolver
                .add_declaration_binding(node.id, declaration_id);
        }

        Ok(())
    }

    fn visit_path(&mut self, node: SyntaxReader) -> Result<Self::PathOutput, CompileError> {
        let path = node.left_child()?;
        let path_segments = path.multiple_children()?;

        let mut current_declaration_id = DeclarationId(0);
        let mut current_scope_id = self.current_scope_id;

        for segment in path_segments {
            let symbol = self.resolver.create_symbol(&segment, self.source);
            let (next_declaration_id, next_declaration) = self.resolver.find_declaration_in_scope(
                symbol,
                &segment,
                current_scope_id,
                None,
                true,
            )?;

            current_declaration_id = next_declaration_id;
            current_scope_id = next_declaration.scope_id;
        }

        self.resolver
            .add_declaration_binding(node.id, current_declaration_id);

        Ok(current_declaration_id)
    }
}
