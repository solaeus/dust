use smallvec::{SmallVec, smallvec};
use tracing::debug;

use crate::{
    compiler::error::{CompileError, InternalCompileError},
    resolver::{
        Resolver,
        declaration_graph::{Declaration, DeclarationId, DeclarationKind},
        scope_graph::{Scope, ScopeId, ScopeKind},
    },
    source::{Source, SourceFileId},
    syntax::{Syntax, SyntaxId, SyntaxKind, SyntaxReader, SyntaxVisitor},
};

pub struct DeclarationBinder<'a> {
    source: &'a Source<'a>,

    syntax: &'a Syntax,

    resolver: &'a mut Resolver,

    current_scope_id: ScopeId,
}

impl<'a> DeclarationBinder<'a> {
    pub fn new(source: &'a Source, syntax: &'a Syntax, resolver: &'a mut Resolver) -> Self {
        Self {
            source,
            syntax,
            resolver,
            current_scope_id: ScopeId::NONE,
        }
    }

    pub fn bind_main(mut self) -> Result<DeclarationId, CompileError> {
        let main_root = self
            .syntax
            .get_tree(SourceFileId::MAIN)
            .ok_or(CompileError::Internal(
                InternalCompileError::MissingSyntaxTree(SourceFileId::MAIN),
            ))?
            .root()
            .ok_or(CompileError::Internal(
                InternalCompileError::MissingSyntaxNode(SyntaxId::ROOT),
            ))?;

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
        debug!("Declaring main function");
        debug_assert_eq!(node.kind(), SyntaxKind::MainFunctionItem);

        let children = node.multiple_children()?;

        let main_symbol_id = self.resolver.symbols.add_anonymous_symbol();
        let main_scope_id = self.resolver.scopes.add_scope(Scope {
            kind: ScopeKind::Function,
            parent: self.current_scope_id,
            modules: smallvec![ScopeId::CORE],
            imports: SmallVec::new(),
        });
        let main_declaration_id = self.resolver.declarations.add_declaration(Declaration {
            symbol_id: main_symbol_id,
            kind: DeclarationKind::Function,
            scope_id: main_scope_id,
            is_public: true,
            position: Some(node.position()),
        });

        self.resolver
            .add_declaration_binding(node.id, main_declaration_id);
        self.resolver.add_scope_binding(node.id, main_scope_id);

        let parent_scope_id = self.current_scope_id;
        self.current_scope_id = main_scope_id;

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

        Ok(main_declaration_id)
    }

    fn visit_module_item(&mut self, _: SyntaxReader) -> Result<Self::ItemOutput, CompileError> {
        debug!("Declaring module item");

        todo!()
    }

    fn visit_function_item(
        &mut self,
        node: SyntaxReader,
    ) -> Result<Self::ItemOutput, CompileError> {
        debug!("Declaring function item");

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

        let function_name_str = self
            .source
            .get_file(function_name.file_id())
            .content_str(function_name.span());
        let function_symbol = self.resolver.symbols.add_named_symbol(function_name_str);
        let function_scope_id = self.resolver.scopes.add_scope(Scope {
            kind: ScopeKind::Function,
            parent: self.current_scope_id,
            modules: smallvec![ScopeId::CORE],
            imports: SmallVec::new(),
        });
        let is_public = match node.kind() {
            SyntaxKind::PublicFunctionItem => true,
            SyntaxKind::FunctionItem => false,
            _ => unreachable!(),
        };
        let function_declaration = Declaration {
            symbol_id: function_symbol,
            kind: DeclarationKind::Function,
            scope_id: self.current_scope_id,
            is_public,
            position: Some(signature.position()),
        };
        let function_declaration_id = self
            .resolver
            .declarations
            .add_declaration(function_declaration);

        for value_parameter in value_parameters {
            let parameter_name = value_parameter.left_child().map_err(CompileError::Syntax)?;
            let parameter_name_str = self
                .source
                .get_file(parameter_name.file_id())
                .content_str(parameter_name.span());
            let parameter_declaration = Declaration {
                symbol_id: self.resolver.symbols.add_named_symbol(parameter_name_str),
                kind: DeclarationKind::Local {
                    shadowed: None,
                    is_mutable: false,
                },
                scope_id: function_scope_id,
                is_public: false,
                position: Some(value_parameter.position()),
            };
            let parameter_declaration_id = self
                .resolver
                .declarations
                .add_declaration(parameter_declaration);

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
            DeclarationBinder::new(self.source, self.syntax, self.resolver);

        function_declaration_binder.visit_block_expression(body, ())?;

        Ok(())
    }

    fn visit_use_item(&mut self, _: SyntaxReader) -> Result<Self::ItemOutput, CompileError> {
        debug!("Declaring use item");

        todo!()
    }

    fn visit_struct_item(&mut self, node: SyntaxReader) -> Result<Self::ItemOutput, CompileError> {
        debug!("Declaring struct item");

        let (struct_name, struct_fields_list) = node.binary_children()?;
        let struct_fields = struct_fields_list.multiple_children()?;

        let struct_name_str = self
            .source
            .get_file(struct_name.file_id())
            .content_str(struct_name.span());
        let struct_symbol = self.resolver.symbols.add_named_symbol(struct_name_str);
        let struct_declaration = Declaration {
            symbol_id: struct_symbol,
            kind: DeclarationKind::Type { parent: None },
            scope_id: self.current_scope_id,
            is_public: false,
            position: Some(node.position()),
        };
        let struct_declaration_id = self
            .resolver
            .declarations
            .add_declaration(struct_declaration);

        let mut field_ids = SmallVec::<[DeclarationId; 8]>::new();

        for field in struct_fields {
            let (field_name, field_type) = field.binary_children()?;

            let field_name_str = self
                .source
                .get_file(field_name.file_id())
                .content_str(field_name.span());
            let field_symbol = self.resolver.symbols.add_named_symbol(field_name_str);
            let field_declaration = Declaration {
                symbol_id: field_symbol,
                kind: DeclarationKind::Type {
                    parent: Some(struct_declaration_id),
                },
                scope_id: self.current_scope_id,
                is_public: false,
                position: Some(field.position()),
            };
            let field_declaration_id = self
                .resolver
                .declarations
                .add_declaration(field_declaration);

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
        debug!("Declaring expression statement");

        self.visit_expression(node.left_child()?, ())
    }

    fn visit_let_statement(
        &mut self,
        node: SyntaxReader,
    ) -> Result<Self::StatementOutput, CompileError> {
        debug!("Declaring let statement");
        debug_assert!(matches!(
            node.kind(),
            SyntaxKind::LetStatement | SyntaxKind::LetMutStatement
        ));

        let mut children = node.multiple_children()?;
        let path = children.expect_next()?;
        let path_segment = {
            let mut segments = path.multiple_children()?;

            if segments.len() != 1 {
                todo!("Handle multi-segment paths in let statements");
            }

            segments.next().unwrap()
        };
        let expression_statement = children.expect_next()?;
        let expression = expression_statement.left_child()?;

        self.visit_expression(expression, ())?;

        let path_segment_str = self
            .source
            .get_file(path_segment.file_id())
            .content_str(path_segment.span());
        let symbol = self.resolver.symbols.add_named_symbol(path_segment_str);
        let shadowed = self
            .resolver
            .find_declaration_in_scope(symbol, &path_segment, self.current_scope_id, None, false)
            .map(|(id, _)| id)
            .ok();
        let is_mutable = node.kind() == SyntaxKind::LetMutStatement;
        let declaration = Declaration {
            symbol_id: symbol,
            kind: DeclarationKind::Local {
                shadowed,
                is_mutable,
            },
            scope_id: self.current_scope_id,
            is_public: false,
            position: Some(node.position()),
        };
        let declaration_id = self.resolver.declarations.add_declaration(declaration);

        self.resolver
            .add_declaration_binding(path.id, declaration_id);

        Ok(())
    }

    fn visit_binary_assignment_statement(
        &mut self,
        node: SyntaxReader,
    ) -> Result<Self::StatementOutput, CompileError> {
        debug!("Declaring binary assignment statement");

        let (path, expression) = node.binary_children()?;

        self.visit_path(path)?;
        self.visit_expression(expression, ())?;

        Ok(())
    }

    fn visit_reassignment_statement(
        &mut self,
        node: SyntaxReader,
    ) -> Result<Self::StatementOutput, CompileError> {
        debug!("Declaring reassignment statement");

        let (path, expression_statement) = node.binary_children()?;
        let expression = expression_statement.left_child()?;

        self.visit_path(path)?;
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
        debug!("Declaring list expression");

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
        debug!("Declaring index expression");

        let (list, index) = node.binary_children()?;

        self.visit_expression(list, ())?;
        self.visit_expression(index, ())?;

        Ok(())
    }

    fn visit_path_expression(
        &mut self,
        path_expression: SyntaxReader,
        _: Self::ExpressionInput,
    ) -> Result<Self::ExpressionOutput, CompileError> {
        debug!("Declaring path expression");

        self.visit_path(path_expression.left_child()?)?;

        Ok(())
    }

    fn visit_struct_expression(
        &mut self,
        node: SyntaxReader,
        _: Self::ExpressionInput,
    ) -> Result<Self::ExpressionOutput, CompileError> {
        debug!("Declaring struct expression");

        let (path, fields_list) = node.binary_children()?;
        let fields = fields_list.multiple_children()?;

        self.visit_path(path)?;

        for field in fields {
            let (field_path, field_expression) = field.binary_children()?;

            self.visit_path(field_path)?;
            self.visit_expression(field_expression, ())?;
        }

        Ok(())
    }

    fn visit_block_expression(
        &mut self,
        node: SyntaxReader,
        _: Self::ExpressionInput,
    ) -> Result<Self::ExpressionOutput, CompileError> {
        debug!("Declaring block expression");

        let children = node.multiple_children()?;

        let block_scope_id = self.resolver.scopes.add_scope(Scope {
            kind: ScopeKind::Block,
            parent: self.current_scope_id,
            modules: smallvec![ScopeId::CORE],
            imports: SmallVec::new(),
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
        debug!("Declaring if expression");

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
        debug!("Declaring else expression");

        self.visit_expression(node.left_child()?, ())?;

        Ok(())
    }

    fn visit_math_binary_expression(
        &mut self,
        node: SyntaxReader,
        _: Self::ExpressionInput,
    ) -> Result<Self::ExpressionOutput, CompileError> {
        debug!("Declaring math binary expression");

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
        debug!("Declaring comparison binary expression");

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
        debug!("Declaring logical binary expression");

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
        debug!("Declaring unary negation expression");

        self.visit_expression(node.left_child()?, ())?;

        Ok(())
    }

    fn visit_while_expression(
        &mut self,
        node: SyntaxReader,
        _: Self::ExpressionInput,
    ) -> Result<Self::ExpressionOutput, CompileError> {
        debug!("Declaring while expression");

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
        debug!("Declaring function expression");

        let (signature, body) = node.binary_children()?;
        let value_parameters = signature.left_child()?.multiple_children()?;

        let function_scope_id = self.resolver.scopes.add_scope(Scope {
            kind: ScopeKind::Function,
            parent: self.current_scope_id,
            modules: smallvec![ScopeId::CORE],
            imports: SmallVec::new(),
        });

        let mut parameter_ids = SmallVec::<[DeclarationId; 8]>::new();

        for value_parameter in value_parameters {
            let parameter_path = value_parameter.left_child()?;

            let parameter_id = self.visit_path(parameter_path)?;

            parameter_ids.push(parameter_id);
        }

        let function_symbol = self.resolver.symbols.add_anonymous_symbol();
        let function_declaration = Declaration {
            symbol_id: function_symbol,
            kind: DeclarationKind::Function,
            scope_id: self.current_scope_id,
            is_public: false,
            position: Some(signature.position()),
        };
        let function_declaration_id = self
            .resolver
            .declarations
            .add_declaration(function_declaration);

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
        debug!("Declaring call expression");

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
            debug!("Declaring path type");

            let path = node.left_child()?;

            let declaration_id = self.visit_path(path)?;

            self.resolver
                .add_declaration_binding(node.id, declaration_id);
        }

        Ok(())
    }

    fn visit_path(&mut self, path: SyntaxReader) -> Result<Self::PathOutput, CompileError> {
        debug!("Declaring path");
        debug_assert_eq!(path.kind(), SyntaxKind::Path);

        let path_segments = path.multiple_children()?;

        debug_assert!(!path_segments.is_empty());

        let file = self.source.get_file(path.file_id());

        let mut current_declaration_id = DeclarationId::CORE;
        let mut current_scope_id = self.current_scope_id;

        for segment in path_segments.rev() {
            let segment_str = file.content_str(segment.span());
            let symbol = self.resolver.symbols.add_named_symbol(segment_str);
            let (next_declaration_id, next_declaration) = self.resolver.find_declaration_in_scope(
                symbol,
                &segment,
                current_scope_id,
                None,
                false,
            )?;

            current_declaration_id = next_declaration_id;
            current_scope_id = next_declaration.scope_id;
        }

        self.resolver
            .add_declaration_binding(path.id, current_declaration_id);

        Ok(current_declaration_id)
    }
}
