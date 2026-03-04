#[cfg(test)]
mod tests;

use std::path::Path;

use smallvec::SmallVec;
use tracing::debug;

use crate::{
    compiler::error::CompileError,
    dust_error::ErrorKind,
    resolver::{
        Resolver,
        declaration_graph::{
            Declaration, DeclarationId, DeclarationKind, DeclarationMembers, ModuleKind,
        },
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

    crate_scope_id: ScopeId,
}

impl<'a> DeclarationBinder<'a> {
    pub fn new(
        source: &'a Source<'a>,
        syntax: &'a Syntax,
        resolver: &'a mut Resolver,
        errors: &'a mut Vec<ErrorKind>,
        crate_scope_id: ScopeId,
    ) -> Self {
        Self {
            source,
            syntax,
            resolver,
            errors,
            current_scope_id: crate_scope_id,
            crate_scope_id,
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

        let children = node.children()?;

        for child in children {
            match self.visit_item(child) {
                Ok(()) => {}
                Err(error) => self.errors.push(error),
            }
        }

        Ok(())
    }

    fn visit_module_item(&mut self, module_item: SyntaxReader) -> Result<(), ErrorKind> {
        debug!("Visiting module item");

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
        let syntax = Some((module_name.position(), module_item.id));

        if let Some(module_body) = module_body {
            let module_declaration_id = self.resolver.declarations.add_declaration(Declaration {
                symbol_id: module_symbol_id,
                kind: DeclarationKind::Module {
                    kind: ModuleKind::Inline,
                    inner_scope_id: module_scope_id,
                },
                scope_id: self.current_scope_id,
                is_public,
                syntax,
            });

            self.resolver
                .add_scope_binding(module_body.id, module_scope_id);
            self.resolver
                .add_declaration_binding(module_item.id, module_declaration_id);

            let starting_scope_id = self.current_scope_id;
            self.current_scope_id = module_scope_id;

            for child in module_body.children()? {
                match self.visit_item(child) {
                    Ok(()) => {}
                    Err(error) => self.errors.push(error),
                }
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
                syntax,
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

        let function_name_str = self.source.get_file_content(&function_name.position())?;
        let function_symbol_id = self.resolver.symbols.add_symbol(function_name_str);

        let is_public = function_item.kind() == SyntaxKind::PublicFunctionItem;
        let function_declaration_id = self.resolver.declarations.add_declaration(Declaration {
            symbol_id: function_symbol_id,
            kind: DeclarationKind::Function,
            scope_id: self.current_scope_id,
            is_public,
            syntax: Some((function_name.position(), function_item.id)),
        });

        self.resolver
            .add_declaration_binding(function_name.id, function_declaration_id);
        self.visit_function_expression(function_expression, ())?;

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

    fn visit_struct_item(&mut self, struct_item: SyntaxReader) -> Result<(), ErrorKind> {
        debug!("Visiting struct item");

        let (struct_name, struct_fields) = struct_item.binary_children()?;

        let struct_name_str = self.source.get_file_content(&struct_name.position())?;
        let struct_symbol = self.resolver.symbols.add_symbol(struct_name_str);
        let struct_declaration_id = self
            .resolver
            .declarations
            .next_declaration_id()
            .offset((struct_fields.child_count() / 2) as u32);

        let mut field_ids = SmallVec::<[DeclarationId; 8]>::new();

        for [field_name, field_type] in struct_fields.children()?.array_chunks::<2>() {
            debug!("Visiting struct field");

            let field_name_str = self.source.get_file_content(&field_name.position())?;
            let field_symbol = self.resolver.symbols.add_symbol(field_name_str);
            let field_declaration_id = self.resolver.declarations.add_declaration(Declaration {
                symbol_id: field_symbol,
                kind: DeclarationKind::Type {
                    parent: Some(struct_declaration_id),
                    type_parameters: DeclarationMembers::default(),
                    members: DeclarationMembers::default(),
                },
                scope_id: self.current_scope_id,
                is_public: false,
                syntax: Some((field_name.position(), field_name.id)),
            });

            self.visit_type(field_type)?;
            self.resolver
                .add_declaration_binding(field_name.id, field_declaration_id);
            field_ids.push(field_declaration_id);
        }

        let members = self
            .resolver
            .declarations
            .add_declaration_members(&field_ids);
        let declared_id = self.resolver.declarations.add_declaration(Declaration {
            symbol_id: struct_symbol,
            kind: DeclarationKind::Type {
                parent: None,
                type_parameters: DeclarationMembers::default(),
                members,
            },
            scope_id: self.current_scope_id,
            is_public: false,
            syntax: Some((struct_item.position(), struct_item.id)),
        });

        debug_assert_eq!(declared_id, struct_declaration_id);
        self.resolver
            .add_declaration_binding(struct_name.id, struct_declaration_id);

        Ok(())
    }

    fn visit_enum_item(&mut self, enum_item: SyntaxReader) -> Result<(), ErrorKind> {
        debug!("Visiting enum item");

        let mut children = enum_item.children()?;
        let enum_name = children.expect_next()?;
        let enum_variants = children.expect_next()?;

        let enum_name_str = self.source.get_file_content(&enum_name.position())?;
        let enum_symbol = self.resolver.symbols.add_symbol(enum_name_str);
        let enum_variants_list = enum_variants.children()?;
        let enum_declaration_id = self
            .resolver
            .declarations
            .next_declaration_id()
            .offset(enum_variants_list.len() as u32);

        let mut variant_ids = SmallVec::<[DeclarationId; 8]>::new();

        for variant in enum_variants_list {
            debug!("Visiting enum variant");

            let variant_name = variant.child()?;

            let variant_name_str = self.source.get_file_content(&variant_name.position())?;
            let variant_symbol = self.resolver.symbols.add_symbol(variant_name_str);
            let variant_declaration_id = self.resolver.declarations.add_declaration(Declaration {
                symbol_id: variant_symbol,
                kind: DeclarationKind::Type {
                    parent: Some(enum_declaration_id),
                    type_parameters: DeclarationMembers::default(),
                    members: DeclarationMembers::default(),
                },
                scope_id: self.current_scope_id,
                is_public: false,
                syntax: Some((variant.position(), variant.id)),
            });

            self.resolver
                .add_declaration_binding(variant_name.id, variant_declaration_id);
            variant_ids.push(variant_declaration_id);
        }

        let members = self
            .resolver
            .declarations
            .add_declaration_members(&variant_ids);
        let declared_id = self.resolver.declarations.add_declaration(Declaration {
            symbol_id: enum_symbol,
            kind: DeclarationKind::Type {
                parent: None,
                type_parameters: DeclarationMembers::default(),
                members,
            },
            scope_id: self.current_scope_id,
            is_public: false,
            syntax: Some((enum_item.position(), enum_item.id)),
        });

        debug_assert_eq!(declared_id, enum_declaration_id);
        self.resolver
            .add_declaration_binding(enum_name.id, enum_declaration_id);

        Ok(())
    }

    fn visit_expression_statement(
        &mut self,
        expression_statement: SyntaxReader,
    ) -> Result<(), ErrorKind> {
        debug!("Visiting expression statement");

        self.visit_expression(expression_statement.child()?, ())
    }

    fn visit_let_statement(&mut self, let_statement: SyntaxReader) -> Result<(), ErrorKind> {
        debug!("Visiting let statement");

        let mut children = let_statement.children()?;
        let simple_path = children.expect_next()?;
        let expression = children.expect_next()?;

        let identifier = self.source.get_file_content(&simple_path.position())?;
        let symbol_id = self.resolver.symbols.add_symbol(identifier);
        let shadowed_declaration = self
            .resolver
            .find_declaration_in_scope(symbol_id, self.current_scope_id, &simple_path)
            .ok()
            .map(|(declaration_id, _)| declaration_id);
        let declaration_id = self.resolver.declarations.add_declaration(Declaration {
            symbol_id,
            kind: DeclarationKind::Local {
                shadowed: shadowed_declaration,
            },
            scope_id: self.current_scope_id,
            is_public: false,
            syntax: Some((simple_path.position(), simple_path.id)),
        });

        self.resolver
            .add_declaration_binding(simple_path.id, declaration_id);
        self.visit_expression(expression, ())?;

        Ok(())
    }

    fn visit_binary_assignment_statement(&mut self, node: SyntaxReader) -> Result<(), ErrorKind> {
        debug!("Visiting binary assignment statement");

        let (path, expression) = node.binary_children()?;

        self.visit_path(path, true)?;
        self.visit_expression(expression, ())?;

        Ok(())
    }

    fn visit_reassignment_statement(&mut self, node: SyntaxReader) -> Result<(), ErrorKind> {
        debug!("Visiting reassignment statement");

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
        debug!("Visiting list expression");

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
        debug!("Visiting index expression");

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
        debug!("Visiting path expression");

        let declaration_id = search_path_segments(self, path_expression)?;

        self.resolver
            .add_declaration_binding(path_expression.id, declaration_id);

        Ok(())
    }

    fn visit_struct_expression(
        &mut self,
        node: SyntaxReader,
        _: Self::ExpressionInput,
    ) -> Result<Self::ExpressionOutput, ErrorKind> {
        debug!("Visiting struct expression");

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
                match self.visit_item(child) {
                    Ok(()) => {}
                    Err(error) => self.errors.push(error),
                }
            } else if child.kind().is_statement() {
                match self.visit_statement(child) {
                    Ok(()) => {}
                    Err(error) => self.errors.push(error),
                }
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
        debug!("Visiting math binary expression");

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
        debug!("Visiting comparison binary expression");

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
        debug!("Visiting logical binary expression");

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
        debug!("Visiting unary negation expression");

        self.visit_expression(node.child()?, ())?;

        Ok(())
    }

    fn visit_while_expression(
        &mut self,
        node: SyntaxReader,
        _: Self::ExpressionInput,
    ) -> Result<Self::ExpressionOutput, ErrorKind> {
        debug!("Visiting while expression");

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
        debug!("Visiting function expression");

        let (signature, body) = node.binary_children()?;
        let mut signature_children = signature.children()?;
        let parameters = signature_children.expect_next()?;
        let return_type = signature_children.next();
        let mut parameters_children = parameters.children()?;
        let value_parameters = parameters_children.expect_next()?;
        let type_parameters = parameters_children.next();

        for [parameter_name, parameter_type] in value_parameters.children()?.array_chunks::<2>() {
            debug!("Visiting function parameter");

            let parameter_name_str = self.source.get_file_content(&parameter_name.position())?;
            let parameter_symbol_id = self.resolver.symbols.add_symbol(parameter_name_str);
            let parameter_declaration_id =
                self.resolver.declarations.add_declaration(Declaration {
                    symbol_id: parameter_symbol_id,
                    kind: DeclarationKind::Local { shadowed: None },
                    scope_id: self.current_scope_id,
                    is_public: false,
                    syntax: Some((parameter_name.position(), parameter_name.id)),
                });

            self.resolver
                .add_declaration_binding(parameter_name.id, parameter_declaration_id);
            self.visit_type(parameter_type)?;
        }

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
        debug!("Visiting call expression");

        let (callee, arguments_list) = node.binary_children()?;
        let arguments = arguments_list.children()?;

        self.visit_expression(callee, ())?;

        for argument in arguments {
            self.visit_expression(argument, ())?;
        }

        Ok(())
    }

    fn visit_type(&mut self, node: SyntaxReader) -> Result<Self::TypeOutput, ErrorKind> {
        debug!("Visiting type");

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
        _local: bool,
    ) -> Result<Self::PathOutput, ErrorKind> {
        debug!("Visiting path");

        let declaration_id = search_path_segments(self, path)?;

        self.resolver
            .add_declaration_binding(path.id, declaration_id);

        Ok(declaration_id)
    }
}

fn search_path_segments<'a>(
    binder: &mut DeclarationBinder<'a>,
    path_expression: SyntaxReader,
) -> Result<DeclarationId, ErrorKind> {
    let segments = path_expression.children()?;

    let file = binder.source.get_file(path_expression.file_id())?;

    let mut current_scope_id = binder.current_scope_id;
    let mut current_declaration_id = None;
    let mut parent_declaration_id = None;
    let mut is_first = true;

    for segment in segments {
        let segment_str = file.content_str(segment.span())?;
        let symbol_id = binder.resolver.symbols.add_symbol(segment_str);
        let (next_declaration_id, next_declaration) = binder
            .resolver
            .find_declaration_in_scope(symbol_id, current_scope_id, &segment)
            .or_else(|error| {
                if is_first {
                    binder.resolver.find_declaration_in_scope(
                        symbol_id,
                        binder.crate_scope_id,
                        &segment,
                    )
                } else {
                    Err(error)
                }
            })?;

        current_declaration_id = Some(next_declaration_id);
        current_scope_id =
            if let DeclarationKind::Module { inner_scope_id, .. } = next_declaration.kind {
                inner_scope_id
            } else {
                break;
            };
        is_first = false;

        if let DeclarationKind::Type {
            parent: Some(parent_id),
            ..
        } = next_declaration.kind
        {
            if let Some(parent_declaration_id) = parent_declaration_id
                && parent_id != parent_declaration_id
            {
                return Err(ErrorKind::Compile(CompileError::Undeclared {
                    symbol_id,
                    usage_position: segment.position(),
                }));
            }

            parent_declaration_id = Some(parent_id);
        } else {
            parent_declaration_id = None;
        }
    }

    Ok(current_declaration_id.unwrap())
}
