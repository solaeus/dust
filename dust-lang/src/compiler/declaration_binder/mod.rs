#[cfg(test)]
mod tests;

use smallvec::SmallVec;
use tracing::debug;

use crate::{
    compiler::error::CompileError,
    error::ErrorKind,
    resolver::{
        Resolver,
        declaration_graph::{
            Declaration, DeclarationId, DeclarationKind, DeclarationMembers, ModuleKind, Visibility,
        },
        scope_graph::{Scope, ScopeId, ScopeKind},
        type_graph::{TypeId, TypeNode},
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
    type TypeOutput = TypeId;
    type PathInput = Visibility;
    type PathOutput = DeclarationId;

    fn visit_root(&mut self, root: SyntaxReader) -> Result<Self::RootOutput, CompileError> {
        debug!("Visiting root");
        debug_assert_eq!(root.kind(), SyntaxKind::Root);

        for child in root.children()? {
            match self.visit_item(child) {
                Ok(()) => {}
                Err(error) => self.errors.push(ErrorKind::Compile(error)),
            }
        }

        Ok(())
    }

    fn visit_module_item(&mut self, module_item: SyntaxReader) -> Result<(), CompileError> {
        debug!("Visiting module item");
        debug_assert!(matches!(
            module_item.kind(),
            SyntaxKind::ModuleItem | SyntaxKind::PublicModuleItem
        ),);

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
        let syntax = Some((module_name.position(), module_item.id));

        if let Some(module_body) = module_body {
            let module_declaration_id = self.resolver.declarations.add_declaration(Declaration {
                symbol_id: module_symbol_id,
                kind: DeclarationKind::Module {
                    kind: ModuleKind::Inline,
                    inner_scope_id: module_scope_id,
                },
                scope_id: self.current_scope_id,
                public: module_item.kind() == SyntaxKind::PublicModuleItem,
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
                    Err(error) => self.errors.push(ErrorKind::Compile(error)),
                }
            }

            self.current_scope_id = starting_scope_id;
        } else {
            let module_file_id = self
                .source
                .iter()
                .find_map(|(file_id, file)| {
                    if file
                        .path()?
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
            let module_declaration_id = self.resolver.declarations.add_declaration(Declaration {
                symbol_id: module_symbol_id,
                kind: DeclarationKind::Module {
                    kind: ModuleKind::File {
                        file_id: module_file_id,
                    },
                    inner_scope_id: module_scope_id,
                },
                scope_id: self.current_scope_id,
                public: module_item.kind() == SyntaxKind::PublicModuleItem,
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

    fn visit_function_item(&mut self, function_item: SyntaxReader) -> Result<(), CompileError> {
        debug!("Visiting function item");
        debug_assert!(matches!(
            function_item.kind(),
            SyntaxKind::FunctionItem | SyntaxKind::PublicFunctionItem
        ),);

        let (function_name, function_expression) = function_item.binary_children()?;
        let (signature, body) = function_expression.binary_children()?;
        let mut signature_children = signature.children()?;
        let parameters = signature_children.expect_next()?;
        let mut parameters_children = parameters.children()?;
        let value_parameters = parameters_children.expect_next()?;
        let type_parameters = parameters_children.next();

        let function_scope_id = self.resolver.scopes.add_scope(Scope {
            kind: ScopeKind::Function,
            parent: self.current_scope_id,
            modules: SmallVec::new(),
            imports: SmallVec::new(),
        });
        let mut value_parameter_ids =
            SmallVec::<[TypeId; 4]>::with_capacity(value_parameters.child_count() / 2);

        for [parameter_name, parameter_type] in value_parameters.children()?.array_chunks::<2>() {
            debug!("Visiting function parameter");

            let name_str = self.source.get_file_content(&parameter_name.position())?;
            let symbol_id = self.resolver.symbols.add_symbol(name_str);
            let type_id = self.visit_type(parameter_type)?;
            let declaration_id = self.resolver.declarations.add_declaration(Declaration {
                symbol_id,
                kind: DeclarationKind::Local { type_id },
                scope_id: function_scope_id,
                public: false,
                syntax: Some((parameter_name.position(), parameter_name.id)),
            });

            self.resolver
                .add_declaration_binding(parameter_name.id, declaration_id);
            value_parameter_ids.push(type_id);
        }

        let type_parameter_ids = if let Some(type_parameters) = type_parameters {
            type_parameters
                .children()?
                .map(|type_parameter| -> Result<DeclarationId, CompileError> {
                    let type_parameter_name = type_parameter.child()?;

                    let type_parameter_name_str = self
                        .source
                        .get_file_content(&type_parameter_name.position())?;
                    let type_parameter_symbol_id =
                        self.resolver.symbols.add_symbol(type_parameter_name_str);
                    let type_parameter_declaration_id =
                        self.resolver.declarations.add_declaration(Declaration {
                            symbol_id: type_parameter_symbol_id,
                            kind: DeclarationKind::Type {
                                parent: None,
                                type_parameters: DeclarationMembers::default(),
                                members: DeclarationMembers::default(),
                            },
                            scope_id: self.current_scope_id,
                            public: false,
                            syntax: Some((type_parameter_name.position(), type_parameter_name.id)),
                        });

                    self.resolver.add_declaration_binding(
                        type_parameter_name.id,
                        type_parameter_declaration_id,
                    );

                    Ok(type_parameter_declaration_id)
                })
                .try_collect::<SmallVec<[DeclarationId; 4]>>()?
        } else {
            SmallVec::new()
        };

        let function_type_id = {
            let type_parameters = self
                .resolver
                .declarations
                .add_declaration_members(&type_parameter_ids);
            let value_parameters = self.resolver.types.add_type_members(&value_parameter_ids);
            let return_type_id = self.resolver.types.create_inferred_type();

            self.resolver.types.add_type(TypeNode::Function {
                type_parameters,
                value_parameters,
                return_type_id,
            })
        };

        let function_name_str = self.source.get_file_content(&function_name.position())?;
        let function_symbol_id = self.resolver.symbols.add_symbol(function_name_str);

        let function_declaration_id = self.resolver.declarations.add_declaration(Declaration {
            symbol_id: function_symbol_id,
            kind: DeclarationKind::Function {
                type_id: function_type_id,
            },
            scope_id: self.current_scope_id,
            public: function_item.kind() == SyntaxKind::PublicFunctionItem,
            syntax: Some((function_item.position(), function_item.id)),
        });

        self.resolver
            .add_declaration_binding(function_name.id, function_declaration_id);
        self.resolver.add_scope_binding(body.id, function_scope_id);

        let starting_scope_id = self.current_scope_id;
        self.current_scope_id = function_scope_id;

        self.visit_block_expression(body, None)?;

        self.current_scope_id = starting_scope_id;

        Ok(())
    }

    fn visit_use_item(&mut self, use_item: SyntaxReader) -> Result<(), CompileError> {
        debug!("Visiting use item");
        debug_assert!(matches!(
            use_item.kind(),
            SyntaxKind::UseItem | SyntaxKind::PublicUseItem
        ),);

        let path = use_item.child()?;
        let path_declaration_id = self.visit_path(path, Visibility::Module)?;
        let path_declaration = self
            .resolver
            .declarations
            .get_declaration(path_declaration_id)?;

        let use_declaration_id = self.resolver.declarations.add_declaration(Declaration {
            symbol_id: path_declaration.symbol_id,
            kind: path_declaration.kind,
            scope_id: self.current_scope_id,
            public: use_item.kind() == SyntaxKind::PublicUseItem,
            syntax: Some((use_item.position(), use_item.id)),
        });

        self.resolver
            .add_declaration_binding(use_item.id, use_declaration_id);

        Ok(())
    }

    fn visit_struct_item(&mut self, struct_item: SyntaxReader) -> Result<(), CompileError> {
        debug!("Visiting struct item");
        debug_assert!(matches!(
            struct_item.kind(),
            SyntaxKind::StructItem | SyntaxKind::PublicStructItem
        ),);

        let (struct_name, struct_fields) = struct_item.binary_children()?;

        let struct_name_str = self.source.get_file_content(&struct_name.position())?;
        let struct_symbol = self.resolver.symbols.add_symbol(struct_name_str);
        let struct_declaration_id = self
            .resolver
            .declarations
            .next_declaration_id()
            .offset((struct_fields.child_count() / 2) as u32);

        let mut field_ids =
            SmallVec::<[DeclarationId; 8]>::with_capacity(struct_fields.child_count() / 2);

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
                public: false,
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
            public: struct_item.kind() == SyntaxKind::PublicStructItem,
            syntax: Some((struct_item.position(), struct_item.id)),
        });

        debug_assert_eq!(declared_id, struct_declaration_id);

        self.resolver
            .add_declaration_binding(struct_name.id, struct_declaration_id);

        Ok(())
    }

    fn visit_enum_item(&mut self, enum_item: SyntaxReader) -> Result<(), CompileError> {
        debug!("Visiting enum item");
        debug_assert!(matches!(
            enum_item.kind(),
            SyntaxKind::EnumItem | SyntaxKind::PublicEnumItem
        ),);

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
                public: false,
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
            public: enum_item.kind() == SyntaxKind::PublicEnumItem,
            syntax: Some((enum_item.position(), enum_item.id)),
        });

        debug_assert_eq!(declared_id, enum_declaration_id);
        self.resolver
            .add_declaration_binding(enum_name.id, enum_declaration_id);

        Ok(())
    }

    fn visit_let_statement(&mut self, let_statement: SyntaxReader) -> Result<(), CompileError> {
        debug!("Visiting let statement");
        debug_assert!(matches!(
            let_statement.kind(),
            SyntaxKind::LetStatement | SyntaxKind::LetMutStatement
        ),);

        let mut children = let_statement.children()?;
        let simple_path = children.expect_next()?;
        let expression = children.expect_next()?;
        let type_notation = children.next();

        self.visit_expression(expression, None)?;

        let identifier = self.source.get_file_content(&simple_path.position())?;
        let symbol_id = self.resolver.symbols.add_symbol(identifier);
        let type_id = if let Some(type_notation) = type_notation {
            self.visit_type(type_notation)?
        } else {
            self.resolver.types.create_inferred_type()
        };

        let declaration_id = self.resolver.declarations.add_declaration(Declaration {
            symbol_id,
            kind: DeclarationKind::Local { type_id },
            scope_id: self.current_scope_id,
            public: false,
            syntax: Some((simple_path.position(), simple_path.id)),
        });

        self.resolver
            .add_declaration_binding(simple_path.id, declaration_id);

        Ok(())
    }

    fn visit_expression_statement(
        &mut self,
        expression_statement: SyntaxReader,
    ) -> Result<(), CompileError> {
        debug!("Visiting expression statement");
        debug_assert_eq!(expression_statement.kind(), SyntaxKind::ExpressionStatement);

        self.visit_expression(expression_statement.child()?, None)?;

        Ok(())
    }

    fn visit_assignment_expression(
        &mut self,
        reassignment_statement: SyntaxReader,
    ) -> Result<(), CompileError> {
        debug!("Visiting reassignment statement");
        debug_assert_eq!(
            reassignment_statement.kind(),
            SyntaxKind::AssignmentExpression
        );

        let (left, right) = reassignment_statement.binary_children()?;

        self.visit_expression(left, None)?;
        self.visit_expression(right, None)?;

        Ok(())
    }

    fn visit_compound_assignment_expression(
        &mut self,
        binary_assignment_statement: SyntaxReader,
    ) -> Result<(), CompileError> {
        debug!("Visiting binary assignment statement");
        debug_assert!(matches!(
            binary_assignment_statement.kind(),
            SyntaxKind::AdditionAssignmentExpression
                | SyntaxKind::SubtractionAssignmentExpression
                | SyntaxKind::MultiplicationAssignmentExpression
                | SyntaxKind::DivisionAssignmentExpression
                | SyntaxKind::ModuloAssignmentExpression
        ),);

        let (left, right) = binary_assignment_statement.binary_children()?;

        self.visit_expression(left, None)?;
        self.visit_expression(right, None)?;

        Ok(())
    }

    fn visit_boolean_expression(
        &mut self,
        _: SyntaxReader,
        _: Option<Self::ExpressionInput>,
    ) -> Result<Self::ExpressionOutput, CompileError> {
        Ok(())
    }

    fn visit_byte_expression(
        &mut self,
        _: SyntaxReader,
        _: Option<Self::ExpressionInput>,
    ) -> Result<Self::ExpressionOutput, CompileError> {
        Ok(())
    }

    fn visit_character_expression(
        &mut self,
        _: SyntaxReader,
        _: Option<Self::ExpressionInput>,
    ) -> Result<Self::ExpressionOutput, CompileError> {
        Ok(())
    }

    fn visit_float_expression(
        &mut self,
        _: SyntaxReader,
        _: Option<Self::ExpressionInput>,
    ) -> Result<Self::ExpressionOutput, CompileError> {
        Ok(())
    }

    fn visit_integer_expression(
        &mut self,
        _: SyntaxReader,
        _: Option<Self::ExpressionInput>,
    ) -> Result<Self::ExpressionOutput, CompileError> {
        Ok(())
    }

    fn visit_string_expression(
        &mut self,
        _: SyntaxReader,
        _: Option<Self::ExpressionInput>,
    ) -> Result<Self::ExpressionOutput, CompileError> {
        Ok(())
    }

    fn visit_list_expression(
        &mut self,
        list_expression: SyntaxReader,
        _: Option<Self::ExpressionInput>,
    ) -> Result<Self::ExpressionOutput, CompileError> {
        debug!("Visiting list expression");
        debug_assert_eq!(list_expression.kind(), SyntaxKind::ListExpression);

        for element in list_expression.children()? {
            self.visit_expression(element, None)?;
        }

        Ok(())
    }

    fn visit_index_expression(
        &mut self,
        index_expression: SyntaxReader,
        _: Option<Self::ExpressionInput>,
    ) -> Result<Self::ExpressionOutput, CompileError> {
        debug!("Visiting index expression");
        debug_assert_eq!(index_expression.kind(), SyntaxKind::IndexExpression);

        let (list, index) = index_expression.binary_children()?;

        self.visit_expression(list, None)?;
        self.visit_expression(index, None)?;

        Ok(())
    }

    fn visit_path_expression(
        &mut self,
        path_expression: SyntaxReader,
        _: Option<Self::ExpressionInput>,
    ) -> Result<Self::ExpressionOutput, CompileError> {
        debug!("Visiting path expression");
        debug_assert_eq!(path_expression.kind(), SyntaxKind::PathExpression);

        let declaration_id = search_path_segments(self, path_expression, Visibility::Block)?;

        self.resolver
            .add_declaration_binding(path_expression.id, declaration_id);

        Ok(())
    }

    fn visit_struct_expression(
        &mut self,
        struct_expression: SyntaxReader,
        _: Option<Self::ExpressionInput>,
    ) -> Result<Self::ExpressionOutput, CompileError> {
        debug!("Visiting struct expression");
        debug_assert_eq!(struct_expression.kind(), SyntaxKind::StructExpression);

        let (path, fields) = struct_expression.binary_children()?;

        self.visit_path(path, Visibility::Module)?;

        for field in fields.children()? {
            let (field_path, field_expression) = field.binary_children()?;

            self.visit_simple_path(field_path, Visibility::Block)?;
            self.visit_expression(field_expression, None)?;
        }

        Ok(())
    }

    fn visit_block_expression(
        &mut self,
        block_expression: SyntaxReader,
        _: Option<Self::ExpressionInput>,
    ) -> Result<Self::ExpressionOutput, CompileError> {
        debug!("Visiting block expression");
        debug_assert_eq!(block_expression.kind(), SyntaxKind::BlockExpression);

        let block_scope_id = self.resolver.scopes.add_scope(Scope {
            kind: ScopeKind::Block,
            parent: self.current_scope_id,
            modules: SmallVec::new(),
            imports: SmallVec::new(),
        });
        let parent_scope_id = self.current_scope_id;
        self.current_scope_id = block_scope_id;

        for child in block_expression.children()? {
            if child.kind().is_statement() {
                match self.visit_statement(child) {
                    Ok(_) => {}
                    Err(error) => self.errors.push(ErrorKind::Compile(error)),
                }
            } else {
                self.visit_expression(child, None)?;
            }
        }

        self.current_scope_id = parent_scope_id;

        self.resolver
            .add_scope_binding(block_expression.id, block_scope_id);

        Ok(())
    }

    fn visit_if_expression(
        &mut self,
        if_expression: SyntaxReader,
        _: Option<Self::ExpressionInput>,
    ) -> Result<Self::ExpressionOutput, CompileError> {
        debug!("Visiting if expression");
        debug_assert_eq!(if_expression.kind(), SyntaxKind::IfExpression);

        let mut children = if_expression.children()?;
        let condition = children.expect_next()?;
        let then_branch = children.expect_next()?;
        let else_branch = children.next();

        self.visit_expression(condition, None)?;
        self.visit_block_expression(then_branch, None)?;

        if let Some(else_branch) = else_branch {
            self.visit_block_expression(else_branch, None)?;
        }

        Ok(())
    }

    fn visit_math_expression(
        &mut self,
        math_expression: SyntaxReader,
        _: Option<Self::ExpressionInput>,
    ) -> Result<Self::ExpressionOutput, CompileError> {
        debug!("Visiting math binary expression");
        debug_assert!(matches!(
            math_expression.kind(),
            SyntaxKind::AdditionExpression
                | SyntaxKind::SubtractionExpression
                | SyntaxKind::MultiplicationExpression
                | SyntaxKind::DivisionExpression
                | SyntaxKind::ModuloExpression
        ),);

        let (left_expression, right_expression) = math_expression.binary_children()?;

        self.visit_expression(left_expression, None)?;
        self.visit_expression(right_expression, None)?;

        Ok(())
    }

    fn visit_comparison_expression(
        &mut self,
        comparison_expression: SyntaxReader,
        _: Option<Self::ExpressionInput>,
    ) -> Result<Self::ExpressionOutput, CompileError> {
        debug!("Visiting comparison binary expression");
        debug_assert!(matches!(
            comparison_expression.kind(),
            SyntaxKind::EqualExpression
                | SyntaxKind::NotEqualExpression
                | SyntaxKind::LessThanExpression
                | SyntaxKind::GreaterThanExpression
                | SyntaxKind::LessThanOrEqualExpression
                | SyntaxKind::GreaterThanOrEqualExpression
        ),);

        let (left_expression, right_expression) = comparison_expression.binary_children()?;

        self.visit_expression(left_expression, None)?;
        self.visit_expression(right_expression, None)?;

        Ok(())
    }

    fn visit_logic_expression(
        &mut self,
        logical_expression: SyntaxReader,
        _: Option<Self::ExpressionInput>,
    ) -> Result<Self::ExpressionOutput, CompileError> {
        debug!("Visiting logical binary expression");
        debug_assert!(matches!(
            logical_expression.kind(),
            SyntaxKind::AndExpression | SyntaxKind::OrExpression
        ),);

        let (left_expression, right_expression) = logical_expression.binary_children()?;

        self.visit_expression(left_expression, None)?;
        self.visit_expression(right_expression, None)?;

        Ok(())
    }

    fn visit_negation_expression(
        &mut self,
        negation_expression: SyntaxReader,
        _: Option<Self::ExpressionInput>,
    ) -> Result<Self::ExpressionOutput, CompileError> {
        debug!("Visiting unary negation expression");
        debug_assert_eq!(negation_expression.kind(), SyntaxKind::NegationExpression);

        self.visit_expression(negation_expression.child()?, None)?;

        Ok(())
    }

    fn visit_while_expression(
        &mut self,
        node: SyntaxReader,
        _: Option<Self::ExpressionInput>,
    ) -> Result<Self::ExpressionOutput, CompileError> {
        debug!("Visiting while expression");
        debug_assert_eq!(node.kind(), SyntaxKind::WhileExpression);

        let (condition, body) = node.binary_children()?;

        self.visit_expression(condition, None)?;
        self.visit_block_expression(body, None)?;

        Ok(())
    }

    fn visit_function_expression(
        &mut self,
        node: SyntaxReader,
        _: Option<Self::ExpressionInput>,
    ) -> Result<Self::ExpressionOutput, CompileError> {
        debug!("Visiting function expression");
        debug_assert_eq!(node.kind(), SyntaxKind::FunctionExpression);

        let (signature, body) = node.binary_children()?;
        let mut signature_children = signature.children()?;
        let parameters = signature_children.expect_next()?;
        let mut parameters_children = parameters.children()?;
        let value_parameters = parameters_children.expect_next()?;

        let function_scope_id = self.resolver.scopes.add_scope(Scope {
            kind: ScopeKind::Function,
            parent: self.current_scope_id,
            modules: SmallVec::new(),
            imports: SmallVec::new(),
        });

        for [parameter_name, parameter_type] in value_parameters.children()?.array_chunks::<2>() {
            debug!("Visiting function parameter");

            let name_str = self.source.get_file_content(&parameter_name.position())?;
            let symbol_id = self.resolver.symbols.add_symbol(name_str);
            let type_id = self.visit_type(parameter_type)?;
            let declaration_id = self.resolver.declarations.add_declaration(Declaration {
                symbol_id,
                kind: DeclarationKind::Local { type_id },
                scope_id: function_scope_id,
                public: false,
                syntax: Some((parameter_name.position(), parameter_name.id)),
            });

            self.resolver
                .add_declaration_binding(parameter_name.id, declaration_id);
            self.visit_type(parameter_type)?;
        }

        self.resolver.add_scope_binding(body.id, function_scope_id);

        let starting_scope_id = self.current_scope_id;
        self.current_scope_id = function_scope_id;

        self.visit_block_expression(body, None)?;

        self.current_scope_id = starting_scope_id;

        Ok(())
    }

    fn visit_call_expression(
        &mut self,
        node: SyntaxReader,
        _: Option<Self::ExpressionInput>,
    ) -> Result<Self::ExpressionOutput, CompileError> {
        debug!("Visiting call expression");
        debug_assert_eq!(node.kind(), SyntaxKind::CallExpression);

        let (callee, arguments_list) = node.binary_children()?;
        let arguments = arguments_list.children()?;

        self.visit_expression(callee, None)?;

        for argument in arguments {
            self.visit_expression(argument, None)?;
        }

        Ok(())
    }

    fn visit_type(&mut self, node: SyntaxReader) -> Result<Self::TypeOutput, CompileError> {
        debug!("Visiting type");
        debug_assert!(matches!(
            node.kind(),
            SyntaxKind::AnyType
                | SyntaxKind::BooleanType
                | SyntaxKind::U8Type
                | SyntaxKind::CharacterType
                | SyntaxKind::F64Type
                | SyntaxKind::I64Type
                | SyntaxKind::StringType
                | SyntaxKind::ListType
                | SyntaxKind::FunctionType
                | SyntaxKind::TypePath
        ),);

        let type_id = match node.kind() {
            SyntaxKind::AnyType => self.resolver.types.create_inferred_type(),
            SyntaxKind::BooleanType => TypeId::BOOLEAN,
            SyntaxKind::U8Type => TypeId::U_8,
            SyntaxKind::CharacterType => TypeId::CHARACTER,
            SyntaxKind::F64Type => TypeId::F_64,
            SyntaxKind::I64Type => TypeId::I_64,
            SyntaxKind::StringType => TypeId::STRING,
            SyntaxKind::ListType => {
                let element_type = node.child()?;

                self.visit_type(element_type)?
            }
            SyntaxKind::FunctionType => {
                let mut children = node.children()?;
                let parameters = children.expect_next()?;
                let mut parameters_children = parameters.children()?;
                let value_parameters = parameters_children.expect_next()?;
                let type_parameters = parameters_children.next();
                let return_type = children.expect_next()?;

                let value_parameter_ids = value_parameters
                    .children()?
                    .map(|parameter| {
                        let parameter_type = parameter.child()?;

                        self.visit_type(parameter_type)
                    })
                    .try_collect::<SmallVec<[TypeId; 4]>>()?;
                let type_parameter_ids = if let Some(type_parameters) = type_parameters {
                    type_parameters
                        .children()?
                        .map(|type_parameter| -> Result<DeclarationId, CompileError> {
                            let type_parameter_name = type_parameter.child()?;

                            let type_parameter_name_str = self
                                .source
                                .get_file_content(&type_parameter_name.position())?;
                            let type_parameter_symbol_id =
                                self.resolver.symbols.add_symbol(type_parameter_name_str);
                            let type_parameter_declaration_id =
                                self.resolver.declarations.add_declaration(Declaration {
                                    symbol_id: type_parameter_symbol_id,
                                    kind: DeclarationKind::Type {
                                        parent: None,
                                        type_parameters: DeclarationMembers::default(),
                                        members: DeclarationMembers::default(),
                                    },
                                    scope_id: self.current_scope_id,
                                    public: false,
                                    syntax: Some((
                                        type_parameter_name.position(),
                                        type_parameter_name.id,
                                    )),
                                });

                            self.resolver.add_declaration_binding(
                                type_parameter_name.id,
                                type_parameter_declaration_id,
                            );

                            Ok(type_parameter_declaration_id)
                        })
                        .try_collect::<SmallVec<[DeclarationId; 4]>>()?
                } else {
                    SmallVec::new()
                };

                let type_parameters = self
                    .resolver
                    .declarations
                    .add_declaration_members(&type_parameter_ids);
                let value_parameters = self.resolver.types.add_type_members(&value_parameter_ids);
                let return_type_id = self.visit_type(return_type)?;

                self.resolver.types.add_type(TypeNode::Function {
                    type_parameters,
                    value_parameters,
                    return_type_id,
                })
            }
            SyntaxKind::TypePath => {
                let declaration_id = search_path_segments(self, node, Visibility::Module)?;

                self.resolver
                    .add_declaration_binding(node.id, declaration_id);
                self.resolver.types.create_inferred_type()
            }
            _ => unreachable!(),
        };

        Ok(type_id)
    }

    fn visit_path(
        &mut self,
        path: SyntaxReader,
        visibility: Self::PathInput,
    ) -> Result<Self::PathOutput, CompileError> {
        debug!("Visiting path");
        debug_assert_eq!(path.kind(), SyntaxKind::Path);

        let declaration_id = search_path_segments(self, path, visibility)?;

        self.resolver
            .add_declaration_binding(path.id, declaration_id);

        Ok(declaration_id)
    }

    fn visit_simple_path(
        &mut self,
        simple_path: SyntaxReader,
        visibility: Self::PathInput,
    ) -> Result<Self::PathOutput, CompileError> {
        debug!("Visiting simple path");
        debug_assert_eq!(simple_path.kind(), SyntaxKind::SimplePath);

        let identifier = self.source.get_file_content(&simple_path.position())?;
        let symbol_id = self.resolver.symbols.add_symbol(identifier);
        let (declaration_id, _) = self.resolver.find_declaration_in_scope(
            symbol_id,
            self.current_scope_id,
            visibility,
            &simple_path,
        )?;

        self.resolver
            .add_declaration_binding(simple_path.id, declaration_id);

        Ok(declaration_id)
    }
}

fn search_path_segments<'a>(
    binder: &mut DeclarationBinder<'a>,
    path_expression: SyntaxReader,
    visibility: Visibility,
) -> Result<DeclarationId, CompileError> {
    let segments = path_expression.children()?;

    let file = binder.source.get_file(path_expression.file_id())?;

    let mut current_scope_id = binder.current_scope_id;
    let mut current_declaration_id = None;
    let mut parent_declaration_id = None;

    for segment in segments {
        let segment_str = file.content_str(segment.span())?;
        let symbol_id = binder.resolver.symbols.add_symbol(segment_str);
        let (next_declaration_id, next_declaration) = binder.resolver.find_declaration_in_scope(
            symbol_id,
            current_scope_id,
            visibility,
            &segment,
        )?;

        current_declaration_id = Some(next_declaration_id);
        current_scope_id =
            if let DeclarationKind::Module { inner_scope_id, .. } = next_declaration.kind {
                inner_scope_id
            } else {
                break;
            };

        if let DeclarationKind::Type {
            parent: Some(next_parent_id),
            ..
        } = next_declaration.kind
        {
            if let Some(target_parent_id) = parent_declaration_id
                && next_parent_id != target_parent_id
            {
                return Err(CompileError::Undeclared {
                    symbol_id,
                    usage_position: segment.position(),
                });
            }

            parent_declaration_id = Some(next_parent_id);
        } else {
            parent_declaration_id = None;
        }
    }

    Ok(current_declaration_id.unwrap())
}
