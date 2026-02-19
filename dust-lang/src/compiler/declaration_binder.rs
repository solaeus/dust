use std::{
    fs::File,
    path::{Path, PathBuf},
    str::FromStr,
};

use memmap2::Mmap;
use smallvec::{SmallVec, smallvec};
use tracing::{debug, info};

use crate::{
    compiler::error::{CompileError, InternalCompileError},
    lexer::Lexer,
    parser::{ParseResult, Parser},
    resolver::{
        Resolver,
        declaration_graph::{Declaration, DeclarationId, DeclarationKind, ModuleKind},
        scope_graph::{Scope, ScopeId, ScopeKind},
    },
    source::{Source, SourceFile, SourceFileId},
    syntax::{Syntax, SyntaxId, SyntaxKind, SyntaxReader, SyntaxVisitor},
};

pub struct DeclarationBinder<'src> {
    source: &'src Source<'src>,

    syntax: &'src Syntax,

    resolver: &'src mut Resolver,

    current_scope_id: ScopeId,
}

impl<'src> DeclarationBinder<'src> {
    pub fn new(
        source: &'src Source<'src>,
        syntax: &'src Syntax,
        resolver: &'src mut Resolver,
        project_scope_id: ScopeId,
    ) -> Self {
        Self {
            source,
            syntax,
            resolver,
            current_scope_id: project_scope_id,
        }
    }
}

impl SyntaxVisitor for DeclarationBinder<'_> {
    type RootOutput = ();
    type ItemOutput = ();
    type StatementOutput = ();
    type ExpressionInput = ();
    type ExpressionOutput = ();
    type TypeOutput = ();
    type PathOutput = DeclarationId;

    fn visit_root(&mut self, node: SyntaxReader) -> Result<Self::RootOutput, CompileError> {
        debug!("Visiting root");
        debug_assert_eq!(node.kind(), SyntaxKind::Root);

        let children = node.children();

        for child in children {
            if child.kind().is_item() {
                self.visit_item(child)?;
            } else if child.kind().is_statement() {
                self.visit_statement(child)?;
            } else {
                self.visit_expression(child, ())?;
            }
        }

        Ok(())
    }

    fn visit_module_item(
        &mut self,
        module_item: SyntaxReader,
    ) -> Result<Self::ItemOutput, CompileError> {
        debug!("Visiting module item");
        debug_assert_eq!(module_item.kind(), SyntaxKind::ModuleItem);

        let mut children = module_item.children();
        let module_name = children.expect_next()?;
        let module_body = children.next();

        let module_name_str = self
            .source
            .get_file(module_name.file_id())
            .content_str(module_name.span());
        let module_symbol_id = self.resolver.symbols.add_named_symbol(module_name_str);
        let module_scope_id = self.resolver.scopes.add_scope(Scope {
            kind: ScopeKind::Module,
            parent: self.current_scope_id,
            modules: smallvec![ScopeId::CORE],
            imports: SmallVec::new(),
        });
        let is_public = module_item.kind() == SyntaxKind::PublicModuleItem;
        let position = Some(module_name.position());

        if let Some(module_body) = module_body {
            let module_declaration_id = self.resolver.declarations.add_declaration(Declaration {
                symbol_id: module_symbol_id,
                kind: DeclarationKind::Module {
                    kind: ModuleKind::Inline,
                    inner_scope_id: module_scope_id,
                },
                scope_id: self.current_scope_id,
                is_public,
                position,
            });

            self.resolver
                .add_scope_binding(module_body.id, module_scope_id);
            self.resolver
                .add_declaration_binding(module_item.id, module_declaration_id);

            let starting_scope_id = self.current_scope_id;
            self.current_scope_id = module_scope_id;

            for child in module_body.children() {
                self.visit_item(child)?;
            }

            self.current_scope_id = starting_scope_id;
        } else {
            let module_declaration_id = self.resolver.declarations.add_declaration(Declaration {
                symbol_id: module_symbol_id,
                kind: DeclarationKind::Module {
                    kind: ModuleKind::File,
                    inner_scope_id: module_scope_id,
                },
                scope_id: self.current_scope_id,
                is_public,
                position,
            });

            self.resolver
                .add_declaration_binding(module_item.id, module_declaration_id);

            let module_file_id = self
                .source
                .files_iter()
                .find_map(|(file_id, file)| {
                    let path = Path::new(file.full_path());

                    if path
                        .file_stem()
                        .and_then(|stem| stem.to_str())
                        .map(|stem_str| stem_str == module_name_str)
                        .unwrap_or(false)
                    {
                        Some(file_id)
                    } else {
                        None
                    }
                })
                .ok_or(CompileError::UnresolvedModule {
                    symbol_id: module_symbol_id,
                })?;
            let module_root = self
                .syntax
                .get_tree(module_file_id)
                .and_then(|tree| tree.root())
                .ok_or(CompileError::Internal(
                    InternalCompileError::MissingSyntaxTree(module_file_id),
                ))?;

            let starting_scope_id = self.current_scope_id;
            self.current_scope_id = module_scope_id;

            self.visit_root(module_root)?;

            self.current_scope_id = starting_scope_id;
        }

        Ok(())
    }

    fn visit_function_item(
        &mut self,
        function_item: SyntaxReader,
    ) -> Result<Self::ItemOutput, CompileError> {
        debug!("Visiting function item");

        let (function_name, function_expression) = function_item.expect_binary_children()?;
        let (signature, body) = function_expression.expect_binary_children()?;
        let value_parameter_list = signature.expect_left_child()?;
        let value_parameters = value_parameter_list.children();
        let return_type = if signature.has_right_child() {
            let right = signature.expect_right_child()?;

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
        let is_public = match function_item.kind() {
            SyntaxKind::PublicFunctionItem => true,
            SyntaxKind::FunctionItem => false,
            _ => unreachable!(),
        };
        let function_declaration = Declaration {
            symbol_id: function_symbol,
            kind: DeclarationKind::Function,
            scope_id: self.current_scope_id,
            is_public,
            position: Some(function_item.position()),
        };
        let function_declaration_id = self
            .resolver
            .declarations
            .add_declaration(function_declaration);

        info!("Declaring function \"{function_name_str}\"");

        for value_parameter in value_parameters {
            let parameter_name = value_parameter
                .expect_left_child()
                .map_err(CompileError::Syntax)?;
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

            info!(
                "Declaring parameter \"{parameter_name_str}\" of function \"{function_name_str}\""
            );
        }

        if let Some(type_node) = return_type {
            self.visit_type(type_node)?;
        }

        self.resolver.add_scope_binding(body.id, function_scope_id);
        self.resolver
            .add_declaration_binding(function_item.id, function_declaration_id);

        let starting_scope_id = self.current_scope_id;
        self.current_scope_id = function_scope_id;

        self.visit_block_expression(body, ())?;

        self.current_scope_id = starting_scope_id;

        Ok(())
    }

    fn visit_use_item(&mut self, _: SyntaxReader) -> Result<Self::ItemOutput, CompileError> {
        todo!()
    }

    fn visit_struct_item(&mut self, node: SyntaxReader) -> Result<Self::ItemOutput, CompileError> {
        let (struct_name, struct_fields_list) = node.expect_binary_children()?;
        let struct_fields = struct_fields_list.expect_multiple_children()?;

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
            let (field_name, field_type) = field.expect_binary_children()?;

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
        self.visit_expression(node.expect_left_child()?, ())
    }

    fn visit_let_statement(
        &mut self,
        node: SyntaxReader,
    ) -> Result<Self::StatementOutput, CompileError> {
        debug!("Visiting let statement");
        debug_assert!(matches!(
            node.kind(),
            SyntaxKind::LetStatement | SyntaxKind::LetMutStatement
        ));

        let mut children = node.children();
        let path = children.expect_next()?;
        let path_segment = {
            let mut segments = path.children();

            if segments.len() != 1 {
                todo!("Handle multi-segment paths in let statements");
            }

            segments.next().unwrap()
        };
        let expression = children.expect_next()?;

        self.visit_expression(expression, ())?;

        let path_segment_str = self
            .source
            .get_file(path_segment.file_id())
            .content_str(path_segment.span());
        let symbol = self.resolver.symbols.add_named_symbol(path_segment_str);
        let shadowed = self
            .resolver
            .find_declaration_in_scope(symbol, self.current_scope_id, None, false, &path_segment)
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
        let (path, expression) = node.expect_binary_children()?;

        self.visit_path(path)?;
        self.visit_expression(expression, ())?;

        Ok(())
    }

    fn visit_reassignment_statement(
        &mut self,
        node: SyntaxReader,
    ) -> Result<Self::StatementOutput, CompileError> {
        let (path, expression_statement) = node.expect_binary_children()?;
        let expression = expression_statement.expect_left_child()?;

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
        for element in node.expect_multiple_children()? {
            self.visit_expression(element, ())?;
        }

        Ok(())
    }

    fn visit_index_expression(
        &mut self,
        node: SyntaxReader,
        _: Self::ExpressionInput,
    ) -> Result<Self::ExpressionOutput, CompileError> {
        let (list, index) = node.expect_binary_children()?;

        self.visit_expression(list, ())?;
        self.visit_expression(index, ())?;

        Ok(())
    }

    fn visit_path_expression(
        &mut self,
        path_expression: SyntaxReader,
        _: Self::ExpressionInput,
    ) -> Result<Self::ExpressionOutput, CompileError> {
        self.visit_path(path_expression.expect_left_child()?)?;

        Ok(())
    }

    fn visit_struct_expression(
        &mut self,
        node: SyntaxReader,
        _: Self::ExpressionInput,
    ) -> Result<Self::ExpressionOutput, CompileError> {
        let (path, fields_list) = node.expect_binary_children()?;
        let fields = fields_list.expect_multiple_children()?;

        self.visit_path(path)?;

        for field in fields {
            let (field_path, field_expression) = field.expect_binary_children()?;

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
        debug!("Visiting block expression");

        let children = node.children();

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
        debug!("Visiting if expression");

        let mut children = node.children();
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
        debug!("Visiting else expression");

        self.visit_expression(node.expect_left_child()?, ())?;

        Ok(())
    }

    fn visit_math_binary_expression(
        &mut self,
        node: SyntaxReader,
        _: Self::ExpressionInput,
    ) -> Result<Self::ExpressionOutput, CompileError> {
        let (left_expression, right_expression) = node.expect_binary_children()?;

        self.visit_expression(left_expression, ())?;
        self.visit_expression(right_expression, ())?;

        Ok(())
    }

    fn visit_comparison_binary_expression(
        &mut self,
        node: SyntaxReader,
        _: Self::ExpressionInput,
    ) -> Result<Self::ExpressionOutput, CompileError> {
        let (left_expression, right_expression) = node.expect_binary_children()?;

        self.visit_expression(left_expression, ())?;
        self.visit_expression(right_expression, ())?;

        Ok(())
    }

    fn visit_logical_binary_expression(
        &mut self,
        node: SyntaxReader,
        _: Self::ExpressionInput,
    ) -> Result<Self::ExpressionOutput, CompileError> {
        let (left_expression, right_expression) = node.expect_binary_children()?;

        self.visit_expression(left_expression, ())?;
        self.visit_expression(right_expression, ())?;

        Ok(())
    }

    fn visit_unary_negation_expression(
        &mut self,
        node: SyntaxReader,
        _: Self::ExpressionInput,
    ) -> Result<Self::ExpressionOutput, CompileError> {
        self.visit_expression(node.expect_left_child()?, ())?;

        Ok(())
    }

    fn visit_while_expression(
        &mut self,
        node: SyntaxReader,
        _: Self::ExpressionInput,
    ) -> Result<Self::ExpressionOutput, CompileError> {
        let (condition, body) = node.expect_binary_children()?;

        self.visit_expression(condition, ())?;
        self.visit_block_expression(body, ())?;

        Ok(())
    }

    fn visit_function_expression(
        &mut self,
        node: SyntaxReader,
        _: Self::ExpressionInput,
    ) -> Result<Self::ExpressionOutput, CompileError> {
        let (signature, body) = node.expect_binary_children()?;
        let value_parameters = signature.expect_left_child()?.expect_multiple_children()?;

        let function_scope_id = self.resolver.scopes.add_scope(Scope {
            kind: ScopeKind::Function,
            parent: self.current_scope_id,
            modules: smallvec![ScopeId::CORE],
            imports: SmallVec::new(),
        });

        let mut parameter_ids = SmallVec::<[DeclarationId; 8]>::new();

        for value_parameter in value_parameters {
            let parameter_path = value_parameter.expect_left_child()?;

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
        let (callee, arguments_list) = node.expect_binary_children()?;
        let arguments = arguments_list.expect_multiple_children()?;

        self.visit_expression(callee, ())?;

        for argument in arguments {
            self.visit_expression(argument, ())?;
        }

        Ok(())
    }

    fn visit_type(&mut self, node: SyntaxReader) -> Result<Self::TypeOutput, CompileError> {
        if node.kind() == SyntaxKind::TypePath {
            let path = node.expect_left_child()?;

            let declaration_id = self.visit_path(path)?;

            self.resolver
                .add_declaration_binding(node.id, declaration_id);
        }

        Ok(())
    }

    fn visit_path(&mut self, path: SyntaxReader) -> Result<Self::PathOutput, CompileError> {
        debug_assert_eq!(path.kind(), SyntaxKind::Path);

        let path_segments = path.expect_multiple_children()?;

        debug_assert!(!path_segments.is_empty());

        let file = self.source.get_file(path.file_id());

        let mut current_declaration_id = DeclarationId::CORE;
        let mut current_scope_id = self.current_scope_id;

        for segment in path_segments.rev() {
            let segment_str = file.content_str(segment.span());
            let symbol = self.resolver.symbols.add_named_symbol(segment_str);
            let (next_declaration_id, next_declaration) = self.resolver.find_declaration_in_scope(
                symbol,
                current_scope_id,
                None,
                false,
                &segment,
            )?;

            current_declaration_id = next_declaration_id;
            current_scope_id = next_declaration.scope_id;
        }

        self.resolver
            .add_declaration_binding(path.id, current_declaration_id);

        Ok(current_declaration_id)
    }
}
