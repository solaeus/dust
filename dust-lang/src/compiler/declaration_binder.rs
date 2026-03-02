use std::path::Path;

use smallvec::SmallVec;
use tracing::{debug, info};

use crate::{
    compiler::error::CompileError,
    dust_error::ErrorKind,
    resolver::{
        Resolver,
        declaration_graph::{Declaration, DeclarationId, DeclarationKind, ModuleKind},
        scope_graph::{Scope, ScopeId, ScopeKind},
    },
    source::Source,
    syntax::{Syntax, SyntaxKind, SyntaxReader, SyntaxVisitor},
};

pub struct DeclarationBinder<'a> {
    source: &'a Source<'a>,

    syntax: &'a Syntax,

    resolver: &'a mut Resolver,

    errors: &'a mut Vec<ErrorKind>,

    current_scope_id: ScopeId,
}

impl<'a> DeclarationBinder<'a> {
    pub fn new(
        source: &'a Source<'a>,
        syntax: &'a Syntax,
        resolver: &'a mut Resolver,
        errors: &'a mut Vec<ErrorKind>,
        project_scope_id: ScopeId,
    ) -> Self {
        Self {
            source,
            syntax,
            resolver,
            errors,
            current_scope_id: project_scope_id,
        }
    }
}

impl SyntaxVisitor for DeclarationBinder<'_> {
    type RootOutput = ();
    type StatementOutput = ();
    type ExpressionInput = ();
    type ExpressionOutput = ();
    type TypeOutput = ();
    type PathOutput = DeclarationId;

    fn visit_root(&mut self, node: SyntaxReader) -> Result<Self::RootOutput, ErrorKind> {
        debug!("Visiting root");
        debug_assert_eq!(node.kind(), SyntaxKind::Root);

        let children = node.children()?;

        for child in children {
            self.visit_item(child)
                .unwrap_or_else(|err| self.errors.push(err));
        }

        Ok(())
    }

    fn visit_module_item(&mut self, module_item: SyntaxReader) -> Result<(), ErrorKind> {
        debug!("Visiting module item");
        debug_assert_eq!(module_item.kind(), SyntaxKind::ModuleItem);

        let mut children = module_item.children()?;
        let module_name = children.expect_next()?;
        let module_body = children.next();

        let module_name_str = self
            .source
            .get_file(module_name.file_id())?
            .content_str(module_name.span())?;
        let module_symbol_id = self.resolver.symbols.add_symbol(module_name_str);
        let module_scope_id = self.resolver.scopes.add_scope(Scope {
            kind: ScopeKind::Module,
            parent: self.current_scope_id,
            modules: SmallVec::new(),
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

            for child in module_body.children()? {
                self.visit_item(child);
            }

            self.current_scope_id = starting_scope_id;
        } else {
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
                .ok_or(ErrorKind::Compile(CompileError::UnresolvedModule {
                    symbol_id: module_symbol_id,
                }))?;
            let module_declaration_id = self.resolver.declarations.add_declaration(Declaration {
                symbol_id: module_symbol_id,
                kind: DeclarationKind::Module {
                    kind: ModuleKind::File {
                        file_id: module_file_id,
                    },
                    inner_scope_id: module_scope_id,
                },
                scope_id: self.current_scope_id,
                is_public,
                position,
            });

            self.resolver
                .add_declaration_binding(module_name.id, module_declaration_id);

            let module_root = self.syntax.get_tree(module_file_id)?.root()?;

            let starting_scope_id = self.current_scope_id;
            self.current_scope_id = module_scope_id;

            self.visit_root(module_root)?;

            self.current_scope_id = starting_scope_id;
        }

        Ok(())
    }

    fn visit_function_item(&mut self, function_item: SyntaxReader) -> Result<(), ErrorKind> {
        debug!("Visiting function item");

        let (function_name, function_expression) = function_item.binary_children()?;
        let (signature, body) = function_expression.binary_children()?;
        let mut signature_children = signature.children()?;
        let value_parameter_list = signature_children.expect_next()?;
        let return_type = signature_children.next();
        let value_parameters = value_parameter_list.children()?;

        let function_name_str = self.source.get_file_content(&function_name.position())?;
        let function_symbol = self.resolver.symbols.add_symbol(function_name_str);
        let function_scope_id = self.resolver.scopes.add_scope(Scope {
            kind: ScopeKind::Function,
            parent: self.current_scope_id,
            modules: SmallVec::new(),
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

    fn visit_use_item(&mut self, node: SyntaxReader) -> Result<(), ErrorKind> {
        debug!("Visiting use item");

        let path = node.child()?;

        let declaration_id = self.visit_path(path, false)?;

        self.resolver
            .add_declaration_binding(node.id, declaration_id);

        Ok(())
    }

    fn visit_struct_item(&mut self, node: SyntaxReader) -> Result<(), ErrorKind> {
        let (struct_name, struct_fields_list) = node.binary_children()?;
        let struct_fields = struct_fields_list.children()?;

        let struct_name_str = self.source.get_file_content(&struct_name.position())?;
        let struct_symbol = self.resolver.symbols.add_symbol(struct_name_str);
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

            let field_name_str = self.source.get_file_content(&field_name.position())?;
            let field_symbol = self.resolver.symbols.add_symbol(field_name_str);
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

    fn visit_enum_item(&mut self, node: SyntaxReader) -> Result<(), ErrorKind> {
        debug!("Visiting enum item");

        let mut children = node.children()?;
        let enum_name = children.expect_next()?;
        let enum_variants = children.expect_next()?;

        let enum_name_str = self.source.get_file_content(&enum_name.position())?;
        let enum_symbol = self.resolver.symbols.add_symbol(enum_name_str);
        let enum_declaration = Declaration {
            symbol_id: enum_symbol,
            kind: DeclarationKind::Type { parent: None },
            scope_id: self.current_scope_id,
            is_public: false,
            position: Some(node.position()),
        };
        let enum_declaration_id = self.resolver.declarations.add_declaration(enum_declaration);

        for variant in enum_variants.children()? {
            debug!("Visiting enum variant");

            let variant_name = variant.child()?;

            let variant_name_str = self.source.get_file_content(&variant_name.position())?;
            let variant_symbol = self.resolver.symbols.add_symbol(variant_name_str);
            let variant_declaration = Declaration {
                symbol_id: variant_symbol,
                kind: DeclarationKind::Type {
                    parent: Some(enum_declaration_id),
                },
                scope_id: self.current_scope_id,
                is_public: false,
                position: Some(variant.position()),
            };
            let variant_declaration_id = self
                .resolver
                .declarations
                .add_declaration(variant_declaration);

            self.resolver
                .add_declaration_binding(variant_name.id, variant_declaration_id);
        }

        self.resolver
            .add_declaration_binding(enum_name.id, enum_declaration_id);

        Ok(())
    }

    fn visit_expression_statement(&mut self, node: SyntaxReader) -> Result<(), ErrorKind> {
        self.visit_expression(node.child()?, ())
    }

    fn visit_let_statement(&mut self, node: SyntaxReader) -> Result<(), ErrorKind> {
        debug!("Visiting let statement");
        debug_assert!(matches!(
            node.kind(),
            SyntaxKind::LetStatement | SyntaxKind::LetMutStatement
        ));

        let mut children = node.children()?;
        let simple_path = children.expect_next()?;
        let expression = children.expect_next()?;

        let identifier = self.source.get_file_content(&simple_path.position())?;
        let symbol_id = self.resolver.symbols.add_symbol(identifier);
        let shadowed = self
            .resolver
            .find_declaration_in_scope(symbol_id, self.current_scope_id, None, true, &simple_path)
            .ok()
            .map(|(declaration_id, _)| declaration_id);
        let declaration_id = self.resolver.declarations.add_declaration(Declaration {
            symbol_id,
            kind: DeclarationKind::Local { shadowed },
            scope_id: self.current_scope_id,
            is_public: false,
            position: Some(simple_path.position()),
        });

        self.resolver
            .add_declaration_binding(simple_path.id, declaration_id);
        self.visit_expression(expression, ())?;

        Ok(())
    }

    fn visit_binary_assignment_statement(&mut self, node: SyntaxReader) -> Result<(), ErrorKind> {
        let (path, expression) = node.binary_children()?;

        self.visit_path(path, true)?;
        self.visit_expression(expression, ())?;

        Ok(())
    }

    fn visit_reassignment_statement(&mut self, node: SyntaxReader) -> Result<(), ErrorKind> {
        let (path, expression_statement) = node.binary_children()?;
        let expression = expression_statement.child()?;

        self.visit_path(path, true)?;
        self.visit_expression(expression, ())?;

        Ok(())
    }

    fn visit_boolean_expression(
        &mut self,
        _: SyntaxReader,
        _: Self::ExpressionInput,
    ) -> Result<Self::ExpressionOutput, ErrorKind> {
        Ok(())
    }

    fn visit_byte_expression(
        &mut self,
        _: SyntaxReader,
        _: Self::ExpressionInput,
    ) -> Result<Self::ExpressionOutput, ErrorKind> {
        Ok(())
    }

    fn visit_character_expression(
        &mut self,
        _: SyntaxReader,
        _: Self::ExpressionInput,
    ) -> Result<Self::ExpressionOutput, ErrorKind> {
        Ok(())
    }

    fn visit_float_expression(
        &mut self,
        _: SyntaxReader,
        _: Self::ExpressionInput,
    ) -> Result<Self::ExpressionOutput, ErrorKind> {
        Ok(())
    }

    fn visit_integer_expression(
        &mut self,
        _: SyntaxReader,
        _: Self::ExpressionInput,
    ) -> Result<Self::ExpressionOutput, ErrorKind> {
        Ok(())
    }

    fn visit_string_expression(
        &mut self,
        _: SyntaxReader,
        _: Self::ExpressionInput,
    ) -> Result<Self::ExpressionOutput, ErrorKind> {
        Ok(())
    }

    fn visit_list_expression(
        &mut self,
        node: SyntaxReader,
        _: Self::ExpressionInput,
    ) -> Result<Self::ExpressionOutput, ErrorKind> {
        for element in node.children()? {
            self.visit_expression(element, ())?;
        }

        Ok(())
    }

    fn visit_index_expression(
        &mut self,
        node: SyntaxReader,
        _: Self::ExpressionInput,
    ) -> Result<Self::ExpressionOutput, ErrorKind> {
        let (list, index) = node.binary_children()?;

        self.visit_expression(list, ())?;
        self.visit_expression(index, ())?;

        Ok(())
    }

    fn visit_path_expression(
        &mut self,
        path_expression: SyntaxReader,
        _: Self::ExpressionInput,
    ) -> Result<Self::ExpressionOutput, ErrorKind> {
        self.visit_path(path_expression.child()?, true)?;

        Ok(())
    }

    fn visit_struct_expression(
        &mut self,
        node: SyntaxReader,
        _: Self::ExpressionInput,
    ) -> Result<Self::ExpressionOutput, ErrorKind> {
        let (path, fields) = node.binary_children()?;

        self.visit_path(path, false)?;

        for field in fields.children()? {
            let (field_path, field_expression) = field.binary_children()?;

            self.visit_path(field_path, false)?;
            self.visit_expression(field_expression, ())?;
        }

        Ok(())
    }

    fn visit_block_expression(
        &mut self,
        node: SyntaxReader,
        _: Self::ExpressionInput,
    ) -> Result<Self::ExpressionOutput, ErrorKind> {
        debug!("Visiting block expression");

        let children = node.children()?;

        let block_scope_id = self.resolver.scopes.add_scope(Scope {
            kind: ScopeKind::Block,
            parent: self.current_scope_id,
            modules: SmallVec::new(),
            imports: SmallVec::new(),
        });
        let parent_scope_id = self.current_scope_id;
        self.current_scope_id = block_scope_id;

        for child in children {
            if child.kind().is_item() {
                self.visit_item(child);
            } else if child.kind().is_statement() {
                self.visit_statement(child);
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
    ) -> Result<Self::ExpressionOutput, ErrorKind> {
        debug!("Visiting if expression");

        let mut children = node.children()?;
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
    ) -> Result<Self::ExpressionOutput, ErrorKind> {
        debug!("Visiting else expression");

        self.visit_expression(node.child()?, ())?;

        Ok(())
    }

    fn visit_math_binary_expression(
        &mut self,
        node: SyntaxReader,
        _: Self::ExpressionInput,
    ) -> Result<Self::ExpressionOutput, ErrorKind> {
        let (left_expression, right_expression) = node.binary_children()?;

        self.visit_expression(left_expression, ())?;
        self.visit_expression(right_expression, ())?;

        Ok(())
    }

    fn visit_comparison_binary_expression(
        &mut self,
        node: SyntaxReader,
        _: Self::ExpressionInput,
    ) -> Result<Self::ExpressionOutput, ErrorKind> {
        let (left_expression, right_expression) = node.binary_children()?;

        self.visit_expression(left_expression, ())?;
        self.visit_expression(right_expression, ())?;

        Ok(())
    }

    fn visit_logical_binary_expression(
        &mut self,
        node: SyntaxReader,
        _: Self::ExpressionInput,
    ) -> Result<Self::ExpressionOutput, ErrorKind> {
        let (left_expression, right_expression) = node.binary_children()?;

        self.visit_expression(left_expression, ())?;
        self.visit_expression(right_expression, ())?;

        Ok(())
    }

    fn visit_unary_negation_expression(
        &mut self,
        node: SyntaxReader,
        _: Self::ExpressionInput,
    ) -> Result<Self::ExpressionOutput, ErrorKind> {
        self.visit_expression(node.child()?, ())?;

        Ok(())
    }

    fn visit_while_expression(
        &mut self,
        node: SyntaxReader,
        _: Self::ExpressionInput,
    ) -> Result<Self::ExpressionOutput, ErrorKind> {
        let (condition, body) = node.binary_children()?;

        self.visit_expression(condition, ())?;
        self.visit_block_expression(body, ())?;

        Ok(())
    }

    fn visit_function_expression(
        &mut self,
        node: SyntaxReader,
        _: Self::ExpressionInput,
    ) -> Result<Self::ExpressionOutput, ErrorKind> {
        let (_signature, body) = node.binary_children()?;

        let function_scope_id = self.resolver.scopes.add_scope(Scope {
            kind: ScopeKind::Function,
            parent: self.current_scope_id,
            modules: SmallVec::new(),
            imports: SmallVec::new(),
        });

        self.resolver.add_scope_binding(body.id, function_scope_id);

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
    ) -> Result<Self::ExpressionOutput, ErrorKind> {
        let (callee, arguments_list) = node.binary_children()?;
        let arguments = arguments_list.children()?;

        self.visit_expression(callee, ())?;

        for argument in arguments {
            self.visit_expression(argument, ())?;
        }

        Ok(())
    }

    fn visit_type(&mut self, node: SyntaxReader) -> Result<Self::TypeOutput, ErrorKind> {
        if node.kind() == SyntaxKind::TypePath {
            let path = node.child()?;

            let declaration_id = self.visit_path(path, false)?;

            self.resolver
                .add_declaration_binding(node.id, declaration_id);
        }

        Ok(())
    }

    fn visit_path(
        &mut self,
        path: SyntaxReader,
        local: bool,
    ) -> Result<Self::PathOutput, ErrorKind> {
        debug_assert_eq!(path.kind(), SyntaxKind::Path);

        let file = self.source.get_file(path.file_id())?;

        let mut current_declaration_id = DeclarationId::CORE;
        let mut current_scope_id = self.current_scope_id;

        for segment in path.children()?.rev() {
            let segment_str = file.content_str(segment.span())?;
            let symbol = self.resolver.symbols.add_symbol(segment_str);
            let (next_declaration_id, next_declaration) = self.resolver.find_declaration_in_scope(
                symbol,
                current_scope_id,
                None,
                local,
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
