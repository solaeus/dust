use std::mem::replace;

use smallvec::SmallVec;

use crate::{
    compiler::{
        context::{
            Context,
            declarations::{Declaration, DeclarationId, Definition, ModuleKind, VariantKind},
            scopes::{Barrier, ScopeId},
            symbols::SymbolId,
            types::{Type, TypeId, TypeMembers},
        },
        error::CompileError,
        value_creation::create_usize_from_decimal,
    },
    error::ErrorKind,
    source::{Position, Source},
    syntax::{
        Syntax, SyntaxId,
        components::{
            ArrayExpression, ArrayRepeatExpression, ArrayType, AssignmentExpression,
            BlockExpression, CallExpression, ComparisonExpression, ConstItem, EnumItem,
            EnumItemTupleVariant, EnumNamedFieldsVariant, EnumUnitVariant, ExpressionStatement,
            FieldAccessExpression, FunctionItem, FunctionType, GroupedExpression, IfExpression,
            ImplItem, IndexExpression, LetStatement, LogicExpression, MathExpression,
            MethodCallExpression, ModItem, NamedFields, NegationExpression, NotExpression,
            PathSegment, RangeExpression, Root, StructExpression, StructExpressionStructFields,
            StructItem, SyntaxComponent, TraitItem, TupleFields, TupleType, TypeItem,
            TypeParameter, UseItem, ValueParameters, WhileExpression,
        },
        node::{SyntaxFlags, SyntaxKind},
        reader::SyntaxReader,
    },
};

pub struct DeclarationResolver<'a> {
    source: &'a Source<'a>,

    syntax: &'a Syntax,

    context: &'a mut Context,

    errors: &'a mut Vec<ErrorKind>,

    forward_references: Vec<DeclarationId>,

    current_scope_id: ScopeId,

    outer: OuterDeclaration,
}

impl<'a> DeclarationResolver<'a> {
    pub fn new(
        source: &'a Source<'a>,
        syntax: &'a Syntax,
        context: &'a mut Context,
        errors: &'a mut Vec<ErrorKind>,
        starting_scope_id: ScopeId,
    ) -> Self {
        Self {
            source,
            syntax,
            context,
            errors,
            forward_references: Vec::new(),
            current_scope_id: starting_scope_id,
            outer: OuterDeclaration::Other,
        }
    }

    pub fn visit_root(&mut self, reader: SyntaxReader) -> Result<(), CompileError> {
        let Root { items } = reader.as_component()?;

        for item in items.children() {
            match self.visit_item(item) {
                Ok(_) => {}
                Err(error) => self.errors.push(ErrorKind::Compile(error)),
            }
        }

        for forward_reference_id in self.forward_references.drain(..) {
            let forward_reference = *self
                .context
                .declarations
                .get_declaration(forward_reference_id);

            let resolved_declaration_id = {
                let mut crossed_barriers = SmallVec::<[Barrier; 7]>::new();
                let mut current_scope_id = forward_reference.scope_id;

                loop {
                    if let Some(declaration_id) = self
                        .context
                        .declarations
                        .find_declaration_id(forward_reference.symbol_id, current_scope_id)
                        .copied()
                    {
                        let declaration = self.context.declarations.get_declaration(declaration_id);

                        if crossed_barriers
                            .iter()
                            .any(|scope_kind| scope_kind.is_barrier(&declaration.definition))
                        {
                            return Err(CompileError::Undeclared {
                                symbol_id: forward_reference.symbol_id,
                                usage_position: forward_reference.syntax.unwrap().0,
                            });
                        }

                        break declaration_id;
                    }

                    let scope = self.context.scopes.get_scope(current_scope_id);
                    current_scope_id = if let Some(parent) = scope.parent {
                        parent
                    } else {
                        return Err(CompileError::Undeclared {
                            symbol_id: forward_reference.symbol_id,
                            usage_position: forward_reference.syntax.unwrap().0,
                        });
                    };

                    if !crossed_barriers.contains(&scope.barrier) {
                        crossed_barriers.push(scope.barrier);
                    }
                }
            };

            self.context
                .declarations
                .resolve_forward_reference(forward_reference_id, resolved_declaration_id);
        }

        Ok(())
    }

    fn enter_scope(&mut self, barrier: Barrier) {
        self.current_scope_id = self
            .context
            .scopes
            .enter_scope(barrier, Some(self.current_scope_id));
    }

    fn exit_scope(&mut self) {
        self.current_scope_id = self
            .context
            .scopes
            .exit_scope(self.current_scope_id)
            .unwrap_or_else(|| {
                self.errors
                    .push(ErrorKind::Compile(CompileError::ScopeStackUnderflow));

                self.current_scope_id
            });
    }

    fn add_declaration(
        &mut self,
        symbol_id: SymbolId,
        definition: Definition,
        syntax: Option<(Position, SyntaxId)>,
    ) -> DeclarationId {
        let declaration_id = self.context.declarations.add_declaration(Declaration {
            symbol_id,
            scope_id: self.current_scope_id,
            definition,
            syntax,
        });

        self.context.scopes.add_to_current_namespace(declaration_id);

        declaration_id
    }

    fn reserve_declaration_id(
        &mut self,
        symbol_id: SymbolId,
        syntax: Option<(Position, SyntaxId)>,
    ) -> DeclarationId {
        let declaration_id = self.context.declarations.reserve_declaration_id(
            symbol_id,
            self.current_scope_id,
            syntax,
        );

        self.context.scopes.add_to_current_namespace(declaration_id);

        declaration_id
    }

    fn visit_item(&mut self, reader: SyntaxReader) -> Result<(), CompileError> {
        match reader.node.kind {
            SyntaxKind::ModItem => self.visit_mod_item(reader),
            SyntaxKind::UseItem => self.visit_use_item(reader),
            SyntaxKind::FunctionItem => self.visit_fn_item(reader).map(|_| ()),
            SyntaxKind::StructItem => self.visit_struct_item(reader),
            SyntaxKind::EnumItem => self.visit_enum_item(reader),
            SyntaxKind::ConstItem => self.visit_const_item(reader).map(|_| ()),
            SyntaxKind::TypeItem => self.visit_type_item(reader).map(|_| ()),
            SyntaxKind::ImplItem => self.visit_impl_item(reader),
            SyntaxKind::TraitItem => self.visit_trait_item(reader),
            _ => Err(CompileError::UnexpectedSyntax {
                expected: &[
                    SyntaxKind::ModItem,
                    SyntaxKind::UseItem,
                    SyntaxKind::FunctionItem,
                    SyntaxKind::StructItem,
                    SyntaxKind::EnumItem,
                    SyntaxKind::ConstItem,
                    SyntaxKind::TypeItem,
                    SyntaxKind::ImplItem,
                    SyntaxKind::TraitItem,
                ],
                found: reader.node.kind,
            }),
        }
    }

    fn visit_mod_item(&mut self, reader: SyntaxReader) -> Result<(), CompileError> {
        let ModItem { public, name, body } = reader.as_component()?;

        let module_name_str = self.source.get_content(name.position())?;
        let module_symbol_id = self.context.symbols.add_symbol(module_name_str);

        let module_declaration_id =
            self.reserve_declaration_id(module_symbol_id, Some((name.position(), reader.id)));

        self.enter_scope(Barrier::Module);

        let inner_scope_id = self.current_scope_id;

        if let Some(module_body) = body {
            self.context
                .add_declaration_binding(reader.id, module_declaration_id);

            for child in module_body.children() {
                match self.visit_item(child) {
                    Ok(()) => {}
                    Err(error) => self.errors.push(ErrorKind::Compile(error)),
                }
            }

            self.exit_scope();
            self.context.declarations.set_reserved_declaration(
                module_declaration_id,
                Definition::Module {
                    public,
                    kind: ModuleKind::Inline,
                    inner_scope_id: Some(inner_scope_id),
                },
            );
        } else {
            let module_source_id = self
                .source
                .iter()
                .find_map(|(source_id, file)| {
                    if file
                        .path()?
                        .file_stem()
                        .and_then(|stem| stem.to_str())
                        .map(|stem_str| stem_str == module_name_str)
                        .unwrap_or(false)
                    {
                        Some(source_id)
                    } else {
                        None
                    }
                })
                .ok_or(CompileError::UnresolvedModule {
                    symbol_id: module_symbol_id,
                })?;

            self.context
                .add_declaration_binding(name.id, module_declaration_id);

            let module_root = self.syntax.get_tree(module_source_id)?.read_root()?;

            self.visit_root(module_root)?;
            self.exit_scope();
            self.context.declarations.set_reserved_declaration(
                module_declaration_id,
                Definition::Module {
                    public,
                    kind: ModuleKind::File {
                        source_id: module_source_id,
                    },
                    inner_scope_id: Some(inner_scope_id),
                },
            );
        }

        Ok(())
    }

    fn visit_use_item(&mut self, reader: SyntaxReader) -> Result<(), CompileError> {
        let UseItem { public, path } = reader.as_component()?;

        let file = self.source.get_code(path.source_id());
        let mut path_segments = path.children();

        let first_segment = path_segments.next().ok_or(CompileError::ExpectedSyntax {
            expected: &[SyntaxKind::PathSegment],
        })?;
        let first_segment_str = file.get_str(first_segment.node.span)?;
        let first_symbol_id = self.context.symbols.add_symbol(first_segment_str);
        let Some(first_declaration_id) = self.context.find_visible_declaration(
            first_symbol_id,
            self.current_scope_id,
            first_segment,
        )?
        else {
            return Err(CompileError::Undeclared {
                symbol_id: first_symbol_id,
                usage_position: first_segment.position(),
            });
        };

        let mut current_declaration_id = first_declaration_id;
        let mut current_symbol_id = first_symbol_id;
        let mut current_span = first_segment.node.span;

        for segment in path_segments {
            let segment_str = file.get_str(segment.node.span)?;
            let segment_symbol_id = self.context.symbols.add_symbol(segment_str);
            let segment_declaration_id = self.context.find_member_declaration(
                segment_symbol_id,
                current_declaration_id,
                segment,
            )?;

            current_declaration_id = segment_declaration_id;
            current_symbol_id = segment_symbol_id;
            current_span = segment.node.span;
        }

        let use_declaration_id = self.add_declaration(
            current_symbol_id,
            Definition::Use {
                public,
                source_declaration_id: current_declaration_id,
            },
            Some((Position::new(reader.source_id(), current_span), reader.id)),
        );

        self.context
            .add_declaration_binding(reader.id, use_declaration_id);

        Ok(())
    }

    fn visit_fn_item(&mut self, reader: SyntaxReader) -> Result<DeclarationId, CompileError> {
        let FunctionItem {
            public,
            name,
            type_parameters,
            value_parameters,
            return_type,
            where_clause: _,
            body,
        } = reader.as_component()?;

        let function_name_str = self.source.get_content(name.position())?;
        let function_symbol_id = self.context.symbols.add_symbol(function_name_str);
        let function_declaration_id =
            self.reserve_declaration_id(function_symbol_id, Some((reader.position(), reader.id)));

        self.enter_scope(Barrier::Item);

        let type_parameters_scope_id = if let Some(type_parameters) = type_parameters {
            Some(self.visit_type_parameters(type_parameters)?)
        } else {
            None
        };
        let value_parameters_scope_id = if let Some(value_parameters) = value_parameters {
            let ValueParameters { name_type_pairs } = value_parameters.as_component()?;

            self.enter_scope(Barrier::Members);

            if value_parameters
                .node
                .flags
                .get_flag(SyntaxFlags::SELF_VALUE)
            {
                let OuterDeclaration::Impl {
                    self_declaration_id,
                    self_type_id,
                    ..
                } = self.outer
                else {
                    return Err(CompileError::InvalidContext);
                };

                self.context
                    .add_declaration_binding(value_parameters.id, self_declaration_id);
                self.context
                    .add_type_binding(value_parameters.id, self_type_id);
            }

            for (parameter_name, parameter_type) in name_type_pairs {
                let parameter_name_str = self.source.get_content(parameter_name.position())?;
                let parameter_symbol_id = self.context.symbols.add_symbol(parameter_name_str);
                let parameter_type_id = self.visit_type(parameter_type)?;
                let parameter_declaration_id = self.add_declaration(
                    parameter_symbol_id,
                    Definition::Local {
                        mutable: false,
                        shadowed: None,
                        type_id: parameter_type_id,
                    },
                    Some((parameter_name.position(), parameter_name.id)),
                );

                self.context
                    .add_declaration_binding(parameter_name.id, parameter_declaration_id);
            }

            Some(self.current_scope_id)
        } else {
            None
        };
        let return_type_id = if let Some(return_type) = return_type {
            self.visit_type(return_type)?
        } else {
            TypeId::UNIT
        };

        self.enter_scope(Barrier::Block);

        if let Some(body) = body {
            for child in body.children() {
                if child.node.kind.is_statement() {
                    match self.visit_statement(child) {
                        Ok(_) => {}
                        Err(error) => self.errors.push(ErrorKind::Compile(error)),
                    }
                } else {
                    self.visit_expression(child)?;
                }
            }
        }

        self.exit_scope();

        if value_parameters_scope_id.is_some() {
            self.exit_scope();
        }

        let parent_declaration_id = match self.outer {
            OuterDeclaration::Trait {
                trait_declaration_id,
                ..
            } => Some(trait_declaration_id),
            OuterDeclaration::Impl {
                impl_declaration_id,
                ..
            } => Some(impl_declaration_id),
            OuterDeclaration::Other => None,
        };

        self.exit_scope();
        self.context.declarations.set_reserved_declaration(
            function_declaration_id,
            Definition::Function {
                public,
                parent_impl_or_trait: parent_declaration_id,
                type_parameters: type_parameters_scope_id,
                value_parameters: value_parameters_scope_id,
                return_type_id,
            },
        );
        self.context
            .add_declaration_binding(name.id, function_declaration_id);

        Ok(function_declaration_id)
    }

    fn visit_struct_item(&mut self, reader: SyntaxReader) -> Result<(), CompileError> {
        let StructItem {
            public,
            name,
            type_parameters,
            fields,
            ..
        } = reader.as_component()?;

        let struct_name_str = self.source.get_content(name.position())?;
        let struct_symbol_id = self.context.symbols.add_symbol(struct_name_str);
        let struct_declaration_id =
            self.reserve_declaration_id(struct_symbol_id, Some((reader.position(), reader.id)));

        self.enter_scope(Barrier::Item);

        let type_parameters_scope_id = if let Some(type_parameters) = type_parameters {
            Some(self.visit_type_parameters(type_parameters)?)
        } else {
            None
        };
        let fields = if let Some(fields) = fields.filter(|fields| fields.child_count() > 0) {
            self.enter_scope(Barrier::Members);

            match fields.node.kind {
                SyntaxKind::TupleFields => {
                    let TupleFields { types } = TupleFields::from_reader(&fields)?;

                    for (index, field_type) in types.enumerate() {
                        let public = field_type.node.flags.get_flag(SyntaxFlags::PUBLIC);
                        let field_symbol_id = self.context.symbols.add_index_symbol(index as u32);
                        let field_type_id = self.visit_type(field_type)?;

                        self.add_declaration(
                            field_symbol_id,
                            Definition::Field {
                                public,
                                parent_struct: struct_declaration_id,
                                type_id: field_type_id,
                            },
                            Some((field_type.position(), field_type.id)),
                        );
                    }
                }
                SyntaxKind::NamedFields => {
                    let NamedFields { name_type_pairs } = NamedFields::from_reader(&fields)?;

                    let file = self.source.get_code(name.source_id());

                    for (field_name, field_type) in name_type_pairs {
                        let public = field_name.node.flags.get_flag(SyntaxFlags::PUBLIC);
                        let field_name_str = file.get_str(field_name.node.span)?;
                        let field_symbol_id = self.context.symbols.add_symbol(field_name_str);
                        let field_type_id = self.visit_type(field_type)?;
                        let field_declaration_id = self.add_declaration(
                            field_symbol_id,
                            Definition::Field {
                                public,
                                parent_struct: struct_declaration_id,
                                type_id: field_type_id,
                            },
                            Some((field_name.position(), field_name.id)),
                        );

                        self.context
                            .add_declaration_binding(field_name.id, field_declaration_id);
                    }
                }
                _ => {
                    return Err(CompileError::UnexpectedSyntax {
                        expected: &[SyntaxKind::TupleFields, SyntaxKind::NamedFields],
                        found: fields.node.kind,
                    });
                }
            }

            Some(self.current_scope_id)
        } else {
            None
        };

        if fields.is_some() {
            self.exit_scope();
        }

        self.exit_scope();
        self.context.declarations.set_reserved_declaration(
            struct_declaration_id,
            Definition::StructType {
                public,
                type_parameters: type_parameters_scope_id,
                fields,
            },
        );
        self.context
            .add_declaration_binding(reader.id, struct_declaration_id);

        Ok(())
    }

    fn visit_enum_item(&mut self, reader: SyntaxReader) -> Result<(), CompileError> {
        let EnumItem {
            public,
            name,
            type_parameters,
            variants,
        } = reader.as_component()?;

        let enum_name_str = self.source.get_content(name.position())?;
        let enum_symbol_id = self.context.symbols.add_symbol(enum_name_str);
        let enum_declaration_id =
            self.reserve_declaration_id(enum_symbol_id, Some((reader.position(), reader.id)));

        self.enter_scope(Barrier::Item);

        let type_parameters_scope_id = if let Some(type_parameters) = type_parameters {
            Some(self.visit_type_parameters(type_parameters)?)
        } else {
            None
        };

        self.enter_scope(Barrier::Members);

        for (index, variant) in variants.children().enumerate() {
            self.visit_enum_variant(variant, enum_declaration_id, index as u16)?;
        }

        let variants_scope_id = Some(self.current_scope_id);

        self.exit_scope();
        self.exit_scope();
        self.context.declarations.set_reserved_declaration(
            enum_declaration_id,
            Definition::EnumType {
                public,
                type_parameters: type_parameters_scope_id,
                variants: variants_scope_id,
            },
        );
        self.context
            .add_declaration_binding(name.id, enum_declaration_id);

        Ok(())
    }

    fn visit_type_parameters(
        &mut self,
        type_parameters_reader: SyntaxReader,
    ) -> Result<ScopeId, CompileError> {
        let type_parameters_scope_id = self.current_scope_id;

        for type_parameter in type_parameters_reader.children() {
            let TypeParameter { name, bounds } = type_parameter.as_component()?;

            let type_parameter_name_str = self.source.get_content(name.position())?;
            let type_parameter_symbol_id = self.context.symbols.add_symbol(type_parameter_name_str);

            let bounds_scope_id = if let Some(bounds_reader) = bounds {
                self.enter_scope(Barrier::Members);

                for bound in bounds_reader.children() {
                    let bound_declaration_id = self.visit_path(bound)?;
                    self.context
                        .scopes
                        .add_to_current_namespace(bound_declaration_id);
                }

                let scope_id = self.current_scope_id;

                self.exit_scope();

                Some(scope_id)
            } else {
                None
            };

            let type_parameter_declaration_id = self.add_declaration(
                type_parameter_symbol_id,
                Definition::TypeParameter {
                    is_self: false,
                    bounds: bounds_scope_id,
                },
                Some((type_parameter.position(), type_parameter.id)),
            );

            self.context
                .add_declaration_binding(type_parameter.id, type_parameter_declaration_id);
        }

        Ok(type_parameters_scope_id)
    }

    fn visit_enum_variant(
        &mut self,
        reader: SyntaxReader,
        enum_declaration_id: DeclarationId,
        discriminant: u16,
    ) -> Result<DeclarationId, CompileError> {
        let file = self.source.get_code(reader.source_id());

        match reader.node.kind {
            SyntaxKind::EnumUnitVariant => {
                let EnumUnitVariant { name } = reader.as_component()?;

                let variant_name_str = file.get_str(name.node.span)?;
                let variant_symbol_id = self.context.symbols.add_symbol(variant_name_str);
                let variant_declaration_id = self.add_declaration(
                    variant_symbol_id,
                    Definition::Variant {
                        discriminant,
                        enum_declaration_id,
                        fields: None,
                        kind: VariantKind::Unit,
                    },
                    Some((name.position(), name.id)),
                );

                self.context
                    .add_declaration_binding(name.id, variant_declaration_id);

                Ok(variant_declaration_id)
            }
            SyntaxKind::EnumTupleFieldsVariant => {
                let EnumItemTupleVariant { name, tuple_fields } = reader.as_component()?;
                let TupleFields { types } = tuple_fields.as_component()?;

                let variant_name_str = file.get_str(name.node.span)?;
                let variant_symbol_id = self.context.symbols.add_symbol(variant_name_str);
                let variant_declaration_id = self.reserve_declaration_id(
                    variant_symbol_id,
                    Some((reader.position(), reader.id)),
                );

                self.enter_scope(Barrier::Members);

                for (index, field_type) in types.enumerate() {
                    let symbol_id = self.context.symbols.add_index_symbol(index as u32);
                    let type_id = self.visit_type(field_type)?;
                    self.add_declaration(
                        symbol_id,
                        Definition::Field {
                            public: false,
                            parent_struct: variant_declaration_id,
                            type_id,
                        },
                        Some((field_type.position(), field_type.id)),
                    );
                }

                let fields_scope_id = self.current_scope_id;

                self.exit_scope();
                self.context.declarations.set_reserved_declaration(
                    variant_declaration_id,
                    Definition::Variant {
                        discriminant,
                        enum_declaration_id,
                        fields: Some(fields_scope_id),
                        kind: VariantKind::TupleFields,
                    },
                );
                self.context
                    .add_declaration_binding(name.id, variant_declaration_id);

                Ok(variant_declaration_id)
            }
            SyntaxKind::EnumNamedFieldsVariant => {
                let EnumNamedFieldsVariant { name, named_fields } = reader.as_component()?;
                let NamedFields { name_type_pairs } = named_fields.as_component()?;

                let variant_name_str = file.get_str(name.node.span)?;
                let variant_symbol_id = self.context.symbols.add_symbol(variant_name_str);
                let variant_declaration_id = self
                    .reserve_declaration_id(variant_symbol_id, Some((name.position(), name.id)));

                self.enter_scope(Barrier::Members);

                for (field_name, field_type) in name_type_pairs {
                    let field_name_str = file.get_str(field_name.node.span)?;
                    let field_symbol_id = self.context.symbols.add_symbol(field_name_str);
                    let field_type_id = self.visit_type(field_type)?;
                    self.add_declaration(
                        field_symbol_id,
                        Definition::Field {
                            public: false,
                            parent_struct: variant_declaration_id,
                            type_id: field_type_id,
                        },
                        Some((field_name.position(), field_name.id)),
                    );
                }

                let fields_scope_id = self.current_scope_id;

                self.exit_scope();
                self.context.declarations.set_reserved_declaration(
                    variant_declaration_id,
                    Definition::Variant {
                        discriminant,
                        enum_declaration_id,
                        fields: Some(fields_scope_id),
                        kind: VariantKind::NamedFields,
                    },
                );
                self.context
                    .add_declaration_binding(name.id, variant_declaration_id);

                Ok(variant_declaration_id)
            }
            _ => Err(CompileError::UnexpectedSyntax {
                expected: &[
                    SyntaxKind::EnumUnitVariant,
                    SyntaxKind::EnumTupleFieldsVariant,
                    SyntaxKind::EnumNamedFieldsVariant,
                ],
                found: reader.node.kind,
            }),
        }
    }

    fn visit_const_item(&mut self, reader: SyntaxReader) -> Result<DeclarationId, CompileError> {
        let ConstItem {
            public,
            name,
            type_notation,
            value,
        } = reader.as_component()?;

        if let Some(value) = value {
            self.enter_scope(Barrier::Constant);
            self.visit_expression(value)?;
            self.exit_scope();
        }

        let const_name_str = self.source.get_content(name.position())?;
        let const_symbol_id = self.context.symbols.add_symbol(const_name_str);
        let const_type_id = self.visit_type(type_notation)?;
        let const_declaration_id = self.add_declaration(
            const_symbol_id,
            Definition::Constant {
                public,
                type_id: const_type_id,
            },
            Some((reader.position(), reader.id)),
        );

        self.context
            .add_declaration_binding(name.id, const_declaration_id);

        Ok(const_declaration_id)
    }

    fn visit_type_item(&mut self, reader: SyntaxReader) -> Result<DeclarationId, CompileError> {
        let TypeItem {
            public,
            name,
            type_parameters,
            aliased_type,
        } = reader.as_component()?;

        let Some(aliased_type) = aliased_type else {
            return Err(CompileError::ExpectedValue {
                source_id: reader.source_id(),
                syntax_id: reader.id,
            });
        };

        self.enter_scope(Barrier::Item);

        let type_parameters_scope_id = if let Some(type_parameters) = type_parameters {
            Some(self.visit_type_parameters(type_parameters)?)
        } else {
            None
        };

        let aliased_type_id = self.visit_type(aliased_type)?;

        self.exit_scope();

        let type_alias_name_str = self.source.get_content(name.position())?;
        let type_alias_symbol_id = self.context.symbols.add_symbol(type_alias_name_str);
        let type_alias_declaration_id = self.add_declaration(
            type_alias_symbol_id,
            Definition::TypeAlias {
                public,
                type_parameters: type_parameters_scope_id,
                aliased_type_id,
            },
            Some((reader.position(), reader.id)),
        );

        self.context
            .add_declaration_binding(name.id, type_alias_declaration_id);

        Ok(type_alias_declaration_id)
    }

    fn visit_impl_item(&mut self, reader: SyntaxReader) -> Result<(), CompileError> {
        let ImplItem {
            type_parameters,
            trait_path,
            trait_type_arguments,
            self_name,
            self_type_arguments,
            where_clause: _,
            body,
        } = reader.as_component()?;

        let impl_symbol_id = self.context.symbols.add_impl_symbol(reader.position());
        let impl_declaration_id =
            self.reserve_declaration_id(impl_symbol_id, Some((reader.position(), reader.id)));

        self.enter_scope(Barrier::Item);

        let type_parameters_scope_id = if let Some(type_parameters) = type_parameters {
            Some(self.visit_type_parameters(type_parameters)?)
        } else {
            None
        };

        self.enter_scope(Barrier::Members);

        let trait_declaration_id = if let Some(trait_path) = trait_path {
            Some(self.visit_path(trait_path)?)
        } else {
            None
        };

        let self_declaration_id = self.visit_path(self_name)?;
        let self_type_arguments = if let Some(type_arguments) = self_type_arguments {
            let mut type_argument_ids = TypeId::SmallVec::new();

            for child in type_arguments.children() {
                let type_id = self.visit_type(child)?;

                type_argument_ids.push(type_id);
            }

            self.context.types.add_type_members(type_argument_ids)
        } else {
            TypeMembers::default()
        };
        let self_type_id = self.context.types.add_type(Type::Algebraic {
            declaration_id: self_declaration_id,
            type_arguments: self_type_arguments,
        });

        let previous_outer = replace(
            &mut self.outer,
            OuterDeclaration::Impl {
                self_declaration_id,
                self_type_id,
                impl_declaration_id,
            },
        );

        for child in body.children() {
            match child.node.kind {
                SyntaxKind::FunctionItem => {
                    self.visit_fn_item(child)?;
                }
                SyntaxKind::ConstItem => {
                    self.visit_const_item(child)?;
                }
                SyntaxKind::TypeItem => {
                    let TypeItem {
                        public,
                        name,
                        type_parameters,
                        aliased_type,
                    } = child.as_component()?;
                    let Some(aliased_type) = aliased_type else {
                        return Err(CompileError::ExpectedValue {
                            source_id: child.source_id(),
                            syntax_id: child.id,
                        });
                    };
                    let type_parameters_scope_id = if let Some(type_parameters) = type_parameters {
                        Some(self.visit_type_parameters(type_parameters)?)
                    } else {
                        None
                    };
                    let aliased_type_id = self.visit_type(aliased_type)?;
                    let type_name_str = self.source.get_content(name.position())?;
                    let type_symbol_id = self.context.symbols.add_symbol(type_name_str);
                    let type_declaration_id = self.add_declaration(
                        type_symbol_id,
                        Definition::InherentAssociatedType {
                            public,
                            parent: self_declaration_id,
                            type_parameters: type_parameters_scope_id,
                            aliased_type_id,
                        },
                        Some((child.position(), child.id)),
                    );

                    self.context
                        .add_declaration_binding(name.id, type_declaration_id);
                }
                _ => {
                    return Err(CompileError::UnexpectedSyntax {
                        expected: &[
                            SyntaxKind::FunctionItem,
                            SyntaxKind::ConstItem,
                            SyntaxKind::TypeItem,
                        ],
                        found: child.node.kind,
                    });
                }
            }
        }

        let trait_type_arguments = if let Some(type_arguments) = trait_type_arguments {
            let mut type_argument_ids =
                TypeId::SmallVec::with_capacity(type_arguments.child_count());

            for type_argument in type_arguments.children() {
                let type_argument_id = self.visit_type(type_argument)?;

                type_argument_ids.push(type_argument_id);
            }

            self.context.types.add_type_members(type_argument_ids)
        } else {
            TypeMembers::default()
        };

        self.outer = previous_outer;
        let impl_scope_id = Some(self.current_scope_id);

        self.exit_scope();
        self.exit_scope();

        let definition = if let Some(trait_declaration_id) = trait_declaration_id {
            Definition::TraitImplementation {
                type_parameters: type_parameters_scope_id,
                self_declaration_id,
                self_type_arguments,
                trait_declaration_id,
                trait_type_arguments,
                declarations: impl_scope_id,
            }
        } else {
            Definition::InherentImplementation {
                type_parameters: type_parameters_scope_id,
                self_declaration_id,
                self_type_arguments,
                declarations: impl_scope_id,
            }
        };

        self.context
            .declarations
            .set_reserved_declaration(impl_declaration_id, definition);
        self.context
            .add_declaration_binding(reader.id, impl_declaration_id);
        self.context
            .implementations
            .entry(self_declaration_id)
            .or_default()
            .push(impl_declaration_id);

        Ok(())
    }

    fn visit_trait_item(&mut self, reader: SyntaxReader) -> Result<(), CompileError> {
        let TraitItem {
            public,
            name,
            body,
            type_parameters,
            supertraits,
            where_clause: _,
        } = reader.as_component()?;

        let trait_name_str = self.source.get_content(name.position())?;
        let trait_symbol_id = self.context.symbols.add_symbol(trait_name_str);
        let trait_declaration_id =
            self.reserve_declaration_id(trait_symbol_id, Some((reader.position(), reader.id)));

        self.enter_scope(Barrier::Item);

        let type_parameters_scope_id = if let Some(type_parameters) = type_parameters {
            Some(self.visit_type_parameters(type_parameters)?)
        } else {
            None
        };

        let supertraits_scope_id = if let Some(supertraits) = supertraits {
            self.enter_scope(Barrier::Members);

            for supertrait in supertraits.children() {
                let supertrait_declaration_id = self.visit_path(supertrait)?;

                self.context
                    .scopes
                    .add_to_current_namespace(supertrait_declaration_id);
            }

            Some(self.current_scope_id)
        } else {
            None
        };

        self.enter_scope(Barrier::Members);

        let previous_outer = replace(
            &mut self.outer,
            OuterDeclaration::Trait {
                supertraits_scope_id,
                trait_declaration_id,
            },
        );

        for child in body.children() {
            match child.node.kind {
                SyntaxKind::FunctionItem => {
                    self.visit_fn_item(child)?;
                }
                SyntaxKind::TypeItem => {
                    let TypeItem {
                        public,
                        name,
                        type_parameters,
                        aliased_type,
                    } = child.as_component()?;

                    let type_parameters_scope_id = if let Some(type_parameters) = type_parameters {
                        Some(self.visit_type_parameters(type_parameters)?)
                    } else {
                        None
                    };
                    let aliased_type_id = aliased_type
                        .map(|aliased_type| self.visit_type(aliased_type))
                        .transpose()?;
                    let type_name_str = self.source.get_content(name.position())?;
                    let type_symbol_id = self.context.symbols.add_symbol(type_name_str);
                    let type_declaration_id = self.add_declaration(
                        type_symbol_id,
                        Definition::TraitAssociatedType {
                            public,
                            parent: trait_declaration_id,
                            type_parameters: type_parameters_scope_id,
                            default_aliased_type_id: aliased_type_id,
                        },
                        Some((child.position(), child.id)),
                    );

                    self.context
                        .add_declaration_binding(name.id, type_declaration_id);
                }
                SyntaxKind::ConstItem => {
                    let ConstItem {
                        public: _,
                        name,
                        type_notation,
                        value,
                    } = child.as_component()?;

                    if let Some(value) = value {
                        self.visit_expression(value)?;
                    }

                    let const_name_str = self.source.get_content(name.position())?;
                    let const_symbol_id = self.context.symbols.add_symbol(const_name_str);
                    let const_type_id = self.visit_type(type_notation)?;
                    let const_declaration_id = self.add_declaration(
                        const_symbol_id,
                        Definition::InherentAssociatedConstant {
                            public: false,
                            parent: trait_declaration_id,
                            type_id: const_type_id,
                        },
                        Some((child.position(), child.id)),
                    );

                    self.context
                        .add_declaration_binding(name.id, const_declaration_id);
                }
                _ => {
                    return Err(CompileError::UnexpectedSyntax {
                        expected: &[
                            SyntaxKind::FunctionItem,
                            SyntaxKind::TypeItem,
                            SyntaxKind::ConstItem,
                        ],
                        found: child.node.kind,
                    });
                }
            };
        }

        self.outer = previous_outer;
        let declarations_scope_id = Some(self.current_scope_id);

        self.exit_scope();

        if supertraits_scope_id.is_some() {
            self.exit_scope();
        }

        self.exit_scope();
        self.context.declarations.set_reserved_declaration(
            trait_declaration_id,
            Definition::Trait {
                public,
                type_parameters: type_parameters_scope_id,
                supertraits: supertraits_scope_id,
                declarations: declarations_scope_id,
            },
        );
        self.context
            .add_declaration_binding(name.id, trait_declaration_id);

        Ok(())
    }

    fn visit_statement(&mut self, reader: SyntaxReader) -> Result<(), CompileError> {
        match reader.node.kind {
            SyntaxKind::UseItem => self.visit_use_item(reader),
            SyntaxKind::FunctionItem => self.visit_fn_item(reader).map(|_| ()),
            SyntaxKind::TypeItem => self.visit_type_item(reader).map(|_| ()),
            SyntaxKind::ConstItem => self.visit_const_item(reader).map(|_| ()),
            SyntaxKind::StructItem => self.visit_struct_item(reader),
            SyntaxKind::EnumItem => self.visit_enum_item(reader),
            SyntaxKind::ImplItem => self.visit_impl_item(reader),
            SyntaxKind::TraitItem => self.visit_trait_item(reader),
            SyntaxKind::LetStatement => self.visit_let_statement(reader),
            SyntaxKind::ExpressionStatement => self.visit_expression_statement(reader),
            _ => Err(CompileError::UnexpectedSyntax {
                expected: &[
                    SyntaxKind::ConstItem,
                    SyntaxKind::EnumItem,
                    SyntaxKind::ExpressionStatement,
                    SyntaxKind::FunctionItem,
                    SyntaxKind::ImplItem,
                    SyntaxKind::LetStatement,
                    SyntaxKind::StructItem,
                    SyntaxKind::TraitItem,
                    SyntaxKind::TypeItem,
                    SyntaxKind::UseItem,
                ],
                found: reader.node.kind,
            }),
        }
    }

    fn visit_let_statement(&mut self, reader: SyntaxReader) -> Result<(), CompileError> {
        let LetStatement {
            mutable,
            name,
            expression,
            type_notation,
        } = reader.as_component()?;

        self.visit_expression(expression)?;

        let local_name_str = self.source.get_content(name.position())?;
        let local_symbol_id = self.context.symbols.add_symbol(local_name_str);
        let local_type_id = if let Some(type_notation) = type_notation {
            self.visit_type(type_notation)?
        } else {
            self.context.types.create_inferred_type(None)
        };
        let shadowed = self
            .context
            .declarations
            .find_declaration_id(local_symbol_id, self.current_scope_id)
            .copied();
        let declaration_id = self.add_declaration(
            local_symbol_id,
            Definition::Local {
                mutable,
                shadowed,
                type_id: local_type_id,
            },
            Some((name.position(), name.id)),
        );

        self.context
            .add_declaration_binding(name.id, declaration_id);

        Ok(())
    }

    fn visit_expression_statement(&mut self, reader: SyntaxReader) -> Result<(), CompileError> {
        let ExpressionStatement { expression } = reader.as_component()?;

        self.visit_expression(expression)?;

        Ok(())
    }

    fn visit_expression(&mut self, reader: SyntaxReader) -> Result<(), CompileError> {
        match reader.node.kind {
            SyntaxKind::BooleanExpression => self.visit_boolean_expression(reader),
            SyntaxKind::HexadecimalExpression => self.visit_hexadecimal_expression(reader),
            SyntaxKind::CharacterExpression => self.visit_character_expression(reader),
            SyntaxKind::FloatExpression => self.visit_float_expression(reader),
            SyntaxKind::IntegerExpression => self.visit_integer_expression(reader),
            SyntaxKind::StringExpression => self.visit_string_expression(reader),
            SyntaxKind::ArrayExpression => self.visit_array_expression(reader),
            SyntaxKind::ArrayRepeatExpression => self.visit_array_repeat_expression(reader),
            SyntaxKind::IndexExpression => self.visit_index_expression(reader),
            SyntaxKind::RangeExpression | SyntaxKind::RangeInclusiveExpression => {
                self.visit_range_expression(reader)
            }
            SyntaxKind::PathExpression => self.visit_path_expression(reader),
            SyntaxKind::StructExpression => self.visit_struct_expression(reader),
            SyntaxKind::GroupedExpression => self.visit_grouped_expression(reader),
            SyntaxKind::BlockExpression => self.visit_block_expression(reader),
            SyntaxKind::IfExpression => self.visit_if_expression(reader),
            SyntaxKind::NegationExpression => self.visit_negation_expression(reader),
            SyntaxKind::NotExpression => self.visit_not_expression(reader),
            SyntaxKind::WhileExpression => self.visit_while_expression(reader),
            SyntaxKind::BreakExpression => self.visit_break_expression(reader),
            SyntaxKind::CallExpression => self.visit_call_expression(reader),
            SyntaxKind::MethodCallExpression => self.visit_method_call_expression(reader),
            SyntaxKind::FieldAccessExpression => self.visit_field_access_expression(reader),
            SyntaxKind::AndExpression | SyntaxKind::OrExpression => {
                self.visit_logic_expression(reader)
            }
            SyntaxKind::AdditionExpression
            | SyntaxKind::AdditionAssignmentExpression
            | SyntaxKind::SubtractionExpression
            | SyntaxKind::SubtractionAssignmentExpression
            | SyntaxKind::MultiplicationExpression
            | SyntaxKind::MultiplicationAssignmentExpression
            | SyntaxKind::DivisionExpression
            | SyntaxKind::DivisionAssignmentExpression
            | SyntaxKind::ModuloExpression
            | SyntaxKind::ModuloAssignmentExpression
            | SyntaxKind::ExponentExpression
            | SyntaxKind::ExponentAssignmentExpression => self.visit_math_expression(reader),
            SyntaxKind::AssignmentExpression => self.visit_assignment_expression(reader),
            SyntaxKind::GreaterThanExpression
            | SyntaxKind::LessThanExpression
            | SyntaxKind::GreaterThanOrEqualExpression
            | SyntaxKind::LessThanOrEqualExpression
            | SyntaxKind::EqualExpression
            | SyntaxKind::NotEqualExpression => self.visit_comparison_expression(reader),
            SyntaxKind::SelfExpression => self.visit_self_expression(reader),
            _ => Err(CompileError::UnexpectedSyntax {
                expected: &[
                    SyntaxKind::AdditionExpression,
                    SyntaxKind::AndExpression,
                    SyntaxKind::ArrayExpression,
                    SyntaxKind::ArrayRepeatExpression,
                    SyntaxKind::AssignmentExpression,
                    SyntaxKind::BlockExpression,
                    SyntaxKind::BooleanExpression,
                    SyntaxKind::BreakExpression,
                    SyntaxKind::CallExpression,
                    SyntaxKind::CharacterExpression,
                    SyntaxKind::DivisionExpression,
                    SyntaxKind::EqualExpression,
                    SyntaxKind::ExponentExpression,
                    SyntaxKind::FieldAccessExpression,
                    SyntaxKind::FloatExpression,
                    SyntaxKind::GreaterThanExpression,
                    SyntaxKind::GreaterThanOrEqualExpression,
                    SyntaxKind::GroupedExpression,
                    SyntaxKind::HexadecimalExpression,
                    SyntaxKind::IfExpression,
                    SyntaxKind::IndexExpression,
                    SyntaxKind::IntegerExpression,
                    SyntaxKind::LessThanExpression,
                    SyntaxKind::LessThanOrEqualExpression,
                    SyntaxKind::MethodCallExpression,
                    SyntaxKind::ModuloExpression,
                    SyntaxKind::MultiplicationExpression,
                    SyntaxKind::NegationExpression,
                    SyntaxKind::NotEqualExpression,
                    SyntaxKind::NotExpression,
                    SyntaxKind::OrExpression,
                    SyntaxKind::PathExpression,
                    SyntaxKind::RangeExpression,
                    SyntaxKind::StringExpression,
                    SyntaxKind::StructExpression,
                    SyntaxKind::SubtractionExpression,
                    SyntaxKind::WhileExpression,
                ],
                found: reader.node.kind,
            }),
        }
    }

    fn visit_assignment_expression(&mut self, reader: SyntaxReader) -> Result<(), CompileError> {
        let AssignmentExpression { target, source } = reader.as_component()?;

        self.visit_expression(target)?;
        self.visit_expression(source)?;

        Ok(())
    }

    fn visit_boolean_expression(&mut self, _: SyntaxReader) -> Result<(), CompileError> {
        Ok(())
    }

    fn visit_hexadecimal_expression(&mut self, _: SyntaxReader) -> Result<(), CompileError> {
        Ok(())
    }

    fn visit_character_expression(&mut self, _: SyntaxReader) -> Result<(), CompileError> {
        Ok(())
    }

    fn visit_float_expression(&mut self, _: SyntaxReader) -> Result<(), CompileError> {
        Ok(())
    }

    fn visit_integer_expression(&mut self, _: SyntaxReader) -> Result<(), CompileError> {
        Ok(())
    }

    fn visit_string_expression(&mut self, _: SyntaxReader) -> Result<(), CompileError> {
        Ok(())
    }

    fn visit_array_expression(&mut self, reader: SyntaxReader) -> Result<(), CompileError> {
        let ArrayExpression { elements } = reader.as_component()?;

        for element in elements {
            self.visit_expression(element)?;
        }

        Ok(())
    }

    fn visit_array_repeat_expression(&mut self, reader: SyntaxReader) -> Result<(), CompileError> {
        let ArrayRepeatExpression { element, .. } = reader.as_component()?;

        self.visit_expression(element)?;

        Ok(())
    }

    fn visit_index_expression(&mut self, reader: SyntaxReader) -> Result<(), CompileError> {
        let IndexExpression { collection, index } = reader.as_component()?;

        self.visit_expression(collection)?;
        self.visit_expression(index)?;

        Ok(())
    }

    fn visit_range_expression(&mut self, reader: SyntaxReader) -> Result<(), CompileError> {
        let RangeExpression { start, end } = reader.as_component()?;

        let declaration_id = match reader.node.kind {
            SyntaxKind::RangeExpression => DeclarationId::RANGE,
            SyntaxKind::RangeInclusiveExpression => DeclarationId::RANGE_INCLUSIVE,
            _ => {
                return Err(CompileError::UnexpectedSyntax {
                    expected: &[
                        SyntaxKind::RangeExpression,
                        SyntaxKind::RangeInclusiveExpression,
                    ],
                    found: reader.node.kind,
                });
            }
        };

        self.context
            .add_declaration_binding(reader.id, declaration_id);
        self.visit_expression(start)?;
        self.visit_expression(end)?;

        Ok(())
    }

    fn visit_path_expression(&mut self, reader: SyntaxReader) -> Result<(), CompileError> {
        let declaration_id = self.visit_path(reader)?;

        self.context
            .add_declaration_binding(reader.id, declaration_id);

        Ok(())
    }

    fn visit_struct_expression(&mut self, reader: SyntaxReader) -> Result<(), CompileError> {
        let StructExpression { path, fields } = reader.as_component()?;
        let StructExpressionStructFields {
            name_expression_pairs,
        } = fields.as_component()?;

        let struct_declaration_id = self.visit_path(path)?;
        let struct_declaration = self
            .context
            .declarations
            .get_declaration(struct_declaration_id);
        let Definition::StructType {
            fields: fields_scope_id,
            ..
        } = struct_declaration.definition
        else {
            return Err(CompileError::ExpectedStructDefinition(
                struct_declaration_id,
            ));
        };
        for (field_name, field_value) in name_expression_pairs {
            let field_name_str = self.source.get_content(field_name.position())?;
            let field_symbol_id = self.context.symbols.add_symbol(field_name_str);

            if let Some(fields_scope_id) = fields_scope_id
                && let Some(field_declaration_id) = self
                    .context
                    .declarations
                    .find_declaration_id(field_symbol_id, fields_scope_id)
            {
                self.context
                    .add_declaration_binding(field_name.id, *field_declaration_id);
            }

            self.visit_expression(field_value)?;
        }

        Ok(())
    }

    fn visit_grouped_expression(&mut self, reader: SyntaxReader) -> Result<(), CompileError> {
        let GroupedExpression { expression } = reader.as_component()?;

        if let Some(expression) = expression {
            self.visit_expression(expression)
        } else {
            Ok(())
        }
    }

    fn visit_block_expression(&mut self, reader: SyntaxReader) -> Result<(), CompileError> {
        let BlockExpression { children } = reader.as_component()?;

        self.enter_scope(Barrier::Block);

        for child in children {
            if child.node.kind.is_statement() {
                match self.visit_statement(child) {
                    Ok(_) => {}
                    Err(error) => self.errors.push(ErrorKind::Compile(error)),
                }
            } else {
                self.visit_expression(child)?;
            }
        }

        self.exit_scope();

        Ok(())
    }

    fn visit_if_expression(&mut self, reader: SyntaxReader) -> Result<(), CompileError> {
        let IfExpression {
            condition,
            then_branch,
            else_branch,
        } = reader.as_component()?;

        self.visit_expression(condition)?;
        self.visit_block_expression(then_branch)?;

        if let Some(else_branch) = else_branch {
            match else_branch.node.kind {
                SyntaxKind::BlockExpression => self.visit_block_expression(else_branch)?,
                SyntaxKind::IfExpression => self.visit_if_expression(else_branch)?,
                _ => {
                    return Err(CompileError::UnexpectedSyntax {
                        expected: &[SyntaxKind::BlockExpression, SyntaxKind::IfExpression],
                        found: else_branch.node.kind,
                    });
                }
            }
        }

        Ok(())
    }

    fn visit_math_expression(&mut self, reader: SyntaxReader) -> Result<(), CompileError> {
        let MathExpression { left, right } = reader.as_component()?;

        self.visit_expression(left)?;
        self.visit_expression(right)?;

        Ok(())
    }

    fn visit_comparison_expression(&mut self, reader: SyntaxReader) -> Result<(), CompileError> {
        let ComparisonExpression { left, right } = reader.as_component()?;

        self.visit_expression(left)?;
        self.visit_expression(right)?;

        Ok(())
    }

    fn visit_logic_expression(&mut self, reader: SyntaxReader) -> Result<(), CompileError> {
        let LogicExpression { left, right } = reader.as_component()?;

        self.visit_expression(left)?;
        self.visit_expression(right)?;

        Ok(())
    }

    fn visit_negation_expression(&mut self, reader: SyntaxReader) -> Result<(), CompileError> {
        let NegationExpression { operand } = reader.as_component()?;

        self.visit_expression(operand)?;

        Ok(())
    }

    fn visit_not_expression(&mut self, reader: SyntaxReader) -> Result<(), CompileError> {
        let NotExpression { operand } = reader.as_component()?;

        self.visit_expression(operand)?;

        Ok(())
    }

    fn visit_while_expression(&mut self, reader: SyntaxReader) -> Result<(), CompileError> {
        let WhileExpression { condition, body } = reader.as_component()?;

        self.visit_expression(condition)?;
        self.visit_block_expression(body)?;

        Ok(())
    }

    fn visit_break_expression(&mut self, _: SyntaxReader) -> Result<(), CompileError> {
        Ok(())
    }

    fn visit_call_expression(&mut self, reader: SyntaxReader) -> Result<(), CompileError> {
        let CallExpression { callee, arguments } = reader.as_component()?;

        self.visit_expression(callee)?;

        let callee_declaration_id = *self.context.get_declaration_binding(&callee.id)?;

        self.context
            .add_declaration_binding(reader.id, callee_declaration_id);

        for argument in arguments.children() {
            self.visit_expression(argument)?;
        }

        Ok(())
    }

    fn visit_method_call_expression(&mut self, reader: SyntaxReader) -> Result<(), CompileError> {
        let MethodCallExpression {
            method_parent,
            method: _,
            type_arguments,
            value_arguments,
        } = reader.as_component()?;

        self.visit_expression(method_parent)?;

        if let Some(type_arguments) = type_arguments {
            for type_argument in type_arguments.children() {
                let type_id = self.visit_type(type_argument)?;

                self.context.add_type_binding(type_argument.id, type_id);
            }
        }

        if let Some(value_arguments) = value_arguments {
            for value_argument in value_arguments.children() {
                self.visit_expression(value_argument)?;
            }
        }

        Ok(())
    }

    fn visit_field_access_expression(&mut self, reader: SyntaxReader) -> Result<(), CompileError> {
        let FieldAccessExpression {
            struct_expression, ..
        } = reader.as_component()?;

        self.visit_expression(struct_expression)
    }

    fn visit_type(&mut self, reader: SyntaxReader) -> Result<TypeId, CompileError> {
        match reader.node.kind {
            SyntaxKind::BooleanType => Ok(TypeId::BOOLEAN),
            SyntaxKind::I8Type => Ok(TypeId::I_8),
            SyntaxKind::I16Type => Ok(TypeId::I_16),
            SyntaxKind::I32Type => Ok(TypeId::I_32),
            SyntaxKind::I64Type => Ok(TypeId::I_64),
            SyntaxKind::I128Type => Ok(TypeId::I_128),
            SyntaxKind::ISizeType => Ok(TypeId::I_SIZE),
            SyntaxKind::U8Type => Ok(TypeId::U_8),
            SyntaxKind::U16Type => Ok(TypeId::U_16),
            SyntaxKind::U32Type => Ok(TypeId::U_32),
            SyntaxKind::U64Type => Ok(TypeId::U_64),
            SyntaxKind::U128Type => Ok(TypeId::U_128),
            SyntaxKind::USizeType => Ok(TypeId::U_SIZE),
            SyntaxKind::F32Type => Ok(TypeId::F_32),
            SyntaxKind::F64Type => Ok(TypeId::F_64),
            SyntaxKind::CharacterType => Ok(TypeId::CHARACTER),
            SyntaxKind::ArrayType => {
                let ArrayType {
                    element_type,
                    length,
                } = reader.as_component()?;

                let element_type_id = self.visit_type(element_type)?;
                let length_bytes = self
                    .source
                    .get_code(length.source_id())
                    .get_bytes(length.node.span)?;
                let length = create_usize_from_decimal(length_bytes, length)?;

                Ok(self.context.types.add_type(Type::Array {
                    element_type_id,
                    length,
                }))
            }
            SyntaxKind::TupleType => {
                let TupleType { element_types } = reader.as_component()?;

                let element_type_ids = element_types
                    .map(|element_type| self.visit_type(element_type))
                    .try_collect::<TypeId::SmallVec>()?;
                let element_types = self.context.types.add_type_members(element_type_ids);

                Ok(self.context.types.add_type(Type::Tuple { element_types }))
            }
            SyntaxKind::FunctionType => {
                let FunctionType {
                    value_parameter_types,
                    return_type,
                } = reader.as_component()?;

                let value_parameter_ids = value_parameter_types
                    .children()
                    .map(|parameter_type| self.visit_type(parameter_type))
                    .try_collect::<TypeId::SmallVec>()?;
                let value_parameters = self.context.types.add_type_members(value_parameter_ids);
                let return_type_id = if let Some(return_type) = return_type {
                    self.visit_type(return_type)?
                } else {
                    TypeId::UNIT
                };

                Ok(self.context.types.add_type(Type::Function {
                    value_parameters,
                    return_type_id,
                }))
            }
            SyntaxKind::TypePath => {
                let declaration_id = self.visit_path(reader)?;

                self.context
                    .add_declaration_binding(reader.id, declaration_id);

                let declaration = self.context.declarations.get_declaration(declaration_id);

                match declaration.definition {
                    Definition::StructType { .. } | Definition::EnumType { .. } => {
                        let last_path_segment =
                            reader.last_child()?.ok_or(CompileError::ExpectedSyntax {
                                expected: &[SyntaxKind::PathSegment],
                            })?;
                        let type_arguments =
                            self.visit_path_segment_type_arguments(last_path_segment)?;

                        Ok(self.context.types.add_type(Type::Algebraic {
                            declaration_id,
                            type_arguments,
                        }))
                    }
                    Definition::TypeParameter { .. } => Ok(self
                        .context
                        .types
                        .add_type(Type::Generic { declaration_id })),
                    Definition::TraitAssociatedType {
                        parent: trait_declaration_id,
                        ..
                    } => {
                        let mut segments = reader.children();
                        let first_segment =
                            segments.next().ok_or(CompileError::ExpectedSyntax {
                                expected: &[SyntaxKind::PathSegment],
                            })?;

                        let first_declaration_id =
                            *self.context.get_declaration_binding(&first_segment.id)?;
                        let first_declaration = self
                            .context
                            .declarations
                            .get_declaration(first_declaration_id);

                        let base_type_id = match first_declaration.definition {
                            Definition::TypeParameter { .. } => {
                                self.context.types.add_type(Type::Generic {
                                    declaration_id: first_declaration_id,
                                })
                            }
                            Definition::StructType { .. } | Definition::EnumType { .. } => {
                                let PathSegment { type_arguments } =
                                    first_segment.as_component()?;

                                let type_arguments = if let Some(type_arguments) = type_arguments {
                                    let mut type_ids = TypeId::SmallVec::new();

                                    for argument in type_arguments.children() {
                                        let type_id =
                                            *self.context.get_type_binding(&argument.id)?;

                                        type_ids.push(type_id);
                                    }

                                    self.context.types.add_type_members(type_ids)
                                } else {
                                    TypeMembers::default()
                                };

                                self.context.types.add_type(Type::Algebraic {
                                    declaration_id: first_declaration_id,
                                    type_arguments,
                                })
                            }
                            _ => {
                                return Err(CompileError::ExpectedTypeDeclaration(
                                    first_declaration_id,
                                ));
                            }
                        };

                        Ok(self.context.types.add_type(Type::Projection {
                            base_type_id,
                            trait_declaration_id,
                            associated_declaration_id: declaration_id,
                            trait_type_arguments: TypeMembers::default(),
                        }))
                    }
                    _ => Err(CompileError::ExpectedTypeDeclaration(declaration_id)),
                }
            }
            SyntaxKind::SelfType => match self.outer {
                OuterDeclaration::Impl {
                    self_declaration_id,
                    ..
                } => Ok(self.context.types.add_type(Type::Algebraic {
                    declaration_id: self_declaration_id,
                    type_arguments: TypeMembers::default(),
                })),
                _ => Err(CompileError::SelfTypeOutsideOfImpl {
                    position: reader.position(),
                }),
            },
            _ => Err(CompileError::UnexpectedSyntax {
                expected: &[
                    SyntaxKind::BooleanType,
                    SyntaxKind::I8Type,
                    SyntaxKind::I16Type,
                    SyntaxKind::I32Type,
                    SyntaxKind::I64Type,
                    SyntaxKind::I128Type,
                    SyntaxKind::ISizeType,
                    SyntaxKind::U8Type,
                    SyntaxKind::U16Type,
                    SyntaxKind::U32Type,
                    SyntaxKind::U64Type,
                    SyntaxKind::U128Type,
                    SyntaxKind::USizeType,
                    SyntaxKind::F32Type,
                    SyntaxKind::F64Type,
                    SyntaxKind::CharacterType,
                    SyntaxKind::TupleType,
                    SyntaxKind::FunctionType,
                    SyntaxKind::TypePath,
                    SyntaxKind::SelfType,
                ],
                found: reader.node.kind,
            }),
        }
    }

    fn visit_path(&mut self, path: SyntaxReader) -> Result<DeclarationId, CompileError> {
        let source_code = self.source.get_code(path.source_id());

        let mut path_segments = path.children();
        let Some(first_segment) = path_segments.next() else {
            return Err(CompileError::ExpectedSyntax {
                expected: &[SyntaxKind::PathSegment],
            });
        };

        self.visit_path_segment_type_arguments(first_segment)?;

        let first_segment_str = source_code.get_str(first_segment.node.span)?;
        let first_symbol_id = self.context.symbols.add_symbol(first_segment_str);
        let mut current_declaration_id = if let Some(declaration_id) = self
            .context
            .find_visible_declaration(first_symbol_id, self.current_scope_id, first_segment)?
        {
            declaration_id
        } else {
            let declaration_id = self.add_declaration(
                first_symbol_id,
                Definition::ForwardReference { resolved: None },
                Some((first_segment.position(), first_segment.id)),
            );

            self.forward_references.push(declaration_id);

            declaration_id
        };

        self.context
            .add_declaration_binding(first_segment.id, current_declaration_id);

        for path_segment in path_segments {
            self.visit_path_segment_type_arguments(path_segment)?;

            let segment_str = source_code.get_str(path_segment.node.span)?;
            let segment_symbol_id = self.context.symbols.add_symbol(segment_str);

            current_declaration_id = self.context.find_member_declaration(
                segment_symbol_id,
                current_declaration_id,
                path_segment,
            )?;

            self.context
                .add_declaration_binding(path_segment.id, current_declaration_id);
        }

        self.context
            .add_declaration_binding(path.id, current_declaration_id);

        Ok(current_declaration_id)
    }

    fn visit_path_segment_type_arguments(
        &mut self,
        path_segment: SyntaxReader,
    ) -> Result<TypeMembers, CompileError> {
        let PathSegment { type_arguments } = path_segment.as_component()?;

        if let Some(type_arguments) = type_arguments {
            let mut type_ids = TypeId::SmallVec::new();

            for type_argument in type_arguments.children() {
                let type_id = self.visit_type(type_argument)?;

                self.context.add_type_binding(type_argument.id, type_id);
                type_ids.push(type_id);
            }

            let type_members = self.context.types.add_type_members(type_ids);

            Ok(type_members)
        } else {
            Ok(TypeMembers::default())
        }
    }

    fn visit_self_expression(&mut self, reader: SyntaxReader) -> Result<(), CompileError> {
        let OuterDeclaration::Impl {
            self_declaration_id,
            self_type_id,
            ..
        } = self.outer
        else {
            return Err(CompileError::SelfTypeOutsideOfImpl {
                position: reader.position(),
            });
        };

        self.context
            .add_declaration_binding(reader.id, self_declaration_id);
        self.context.add_type_binding(reader.id, self_type_id);

        Ok(())
    }
}

#[derive(Debug)]
enum OuterDeclaration {
    Other,
    Impl {
        self_declaration_id: DeclarationId,
        self_type_id: TypeId,
        impl_declaration_id: DeclarationId,
    },
    Trait {
        trait_declaration_id: DeclarationId,
        supertraits_scope_id: Option<ScopeId>,
    },
}
