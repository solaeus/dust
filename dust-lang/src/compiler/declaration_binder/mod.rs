#[cfg(test)]
mod tests;

use smallvec::SmallVec;

use crate::{
    compiler::{
        error::CompileError,
        resolver::{
            Resolver,
            declarations::{Declaration, DeclarationId, Definition, ModuleKind},
            scopes::{ScopeId, ScopeKind},
            symbols::SymbolId,
            types::{Type, TypeId, TypeMembers},
        },
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
            FieldAccessExpression, FnItem, FunctionType, GroupedExpression, IfExpression, ImplItem,
            IndexExpression, LetStatement, LogicExpression, MathExpression, ModItem, NamedFields,
            NegationExpression, NotExpression, PathSegment, RangeExpression, Root,
            StructExpression, StructExpressionStructFields, StructItem, SyntaxComponent, TraitItem,
            TupleFields, TupleType, TypeItem, UseItem, ValueParameters, WhileExpression,
        },
        node::{SyntaxFlags, SyntaxKind},
        reader::SyntaxReader,
    },
};

pub struct DeclarationBinder<'a> {
    source: &'a Source<'a>,

    syntax: &'a Syntax,

    resolver: &'a mut Resolver,

    errors: &'a mut Vec<ErrorKind>,

    current_scope_id: ScopeId,

    current_self_type_id: Option<TypeId>,
}

impl<'a> DeclarationBinder<'a> {
    pub fn new(
        source: &'a Source<'a>,
        syntax: &'a Syntax,
        resolver: &'a mut Resolver,
        errors: &'a mut Vec<ErrorKind>,
        starting_scope_id: ScopeId,
    ) -> Self {
        Self {
            source,
            syntax,
            resolver,
            errors,
            current_self_type_id: None,
            current_scope_id: starting_scope_id,
        }
    }

    pub fn bind_root(&mut self, reader: SyntaxReader) -> Result<(), CompileError> {
        let Root { items } = reader.as_component()?;

        for item in items.children() {
            match self.bind_item(item) {
                Ok(_) => {}
                Err(error) => self.errors.push(ErrorKind::Compile(error)),
            }
        }

        Ok(())
    }

    fn enter_scope(&mut self, kind: ScopeKind) {
        self.current_scope_id = self
            .resolver
            .scopes
            .enter_scope(kind, Some(self.current_scope_id));
    }

    fn exit_scope(&mut self) {
        self.current_scope_id = self
            .resolver
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
        let declaration_id = self.resolver.declarations.add_declaration(Declaration {
            symbol_id,
            scope_id: self.current_scope_id,
            definition,
            syntax,
        });

        self.resolver
            .scopes
            .add_to_current_namespace(declaration_id);

        declaration_id
    }

    fn reserve_declaration_id(
        &mut self,
        symbol_id: SymbolId,
        syntax: Option<(Position, SyntaxId)>,
    ) -> DeclarationId {
        let declaration_id = self.resolver.declarations.reserve_declaration_id(
            symbol_id,
            self.current_scope_id,
            syntax,
        );

        self.resolver
            .scopes
            .add_to_current_namespace(declaration_id);

        declaration_id
    }

    fn bind_path_segments(
        &mut self,
        path_expression: SyntaxReader,
    ) -> Result<Option<DeclarationId>, CompileError> {
        let source_code = self.source.get_code(path_expression.source_id())?;

        let mut path_segments = path_expression.children();
        let Some(first_segment) = path_segments.next() else {
            return Ok(None);
        };

        self.bind_path_segment_type_arguments(first_segment)?;

        let first_segment_str = source_code.get_str(first_segment.node.span)?;
        let first_symbol_id = self.resolver.symbols.add_symbol(first_segment_str);
        let Some(mut current_declaration_id) =
            self.find_visible_declaration(first_symbol_id, first_segment)?
        else {
            return Ok(None);
        };

        self.resolver
            .add_declaration_binding(first_segment.id, current_declaration_id);

        for path_segment in path_segments {
            self.bind_path_segment_type_arguments(path_segment)?;

            let segment_str = source_code.get_str(path_segment.node.span)?;
            let segment_symbol_id = self.resolver.symbols.add_symbol(segment_str);

            current_declaration_id = self.find_member_declaration(
                segment_symbol_id,
                current_declaration_id,
                path_segment,
            )?;

            self.resolver
                .add_declaration_binding(path_segment.id, current_declaration_id);
        }

        self.resolver
            .add_declaration_binding(path_expression.id, current_declaration_id);

        Ok(Some(current_declaration_id))
    }

    fn bind_path_segment_type_arguments(
        &mut self,
        path_segment: SyntaxReader,
    ) -> Result<(), CompileError> {
        let PathSegment { type_arguments } = path_segment.as_component()?;

        if let Some(type_arguments) = type_arguments {
            for type_argument in type_arguments.children() {
                self.handle_explicit_type(type_argument)?;
            }
        }

        Ok(())
    }

    fn find_visible_declaration(
        &self,
        symbol_id: SymbolId,
        path_segment: SyntaxReader,
    ) -> Result<Option<DeclarationId>, CompileError> {
        let mut scope_id = Some(self.current_scope_id);
        let mut crossed_scope_kinds = SmallVec::<[ScopeKind; 7]>::new();

        while let Some(current_scope_id) = scope_id {
            if let Some(declaration_id) = self
                .resolver
                .declarations
                .find_declaration_id(symbol_id, current_scope_id)
            {
                let declaration = self
                    .resolver
                    .declarations
                    .get_declaration(*declaration_id)?;

                if crossed_scope_kinds
                    .iter()
                    .any(|scope_kind| scope_kind.is_barrier(&declaration.definition))
                {
                    return Err(CompileError::Undeclared {
                        symbol_id,
                        usage_position: path_segment.position(),
                    });
                }

                return Ok(Some(*declaration_id));
            }

            let scope = self.resolver.scopes.get_scope(current_scope_id);
            scope_id = scope.parent;

            if !crossed_scope_kinds.contains(&scope.kind) {
                crossed_scope_kinds.push(scope.kind);
            }
        }

        Ok(None)
    }

    fn find_member_declaration(
        &self,
        symbol_id: SymbolId,
        parent_declaration_id: DeclarationId,
        path_segment: SyntaxReader,
    ) -> Result<DeclarationId, CompileError> {
        let parent_declaration = self
            .resolver
            .declarations
            .get_declaration(parent_declaration_id)?;
        let primary_member_scope_id = match parent_declaration.definition {
            Definition::Module {
                inner_scope_id: Some(inner_scope_id),
                ..
            }
            | Definition::StructType {
                fields: Some(inner_scope_id),
                ..
            }
            | Definition::EnumType {
                variants: Some(inner_scope_id),
                ..
            }
            | Definition::TraitImplementation {
                declarations: Some(inner_scope_id),
                ..
            }
            | Definition::InherentImplementation {
                declarations: Some(inner_scope_id),
                ..
            } => Some(inner_scope_id),
            Definition::StructType { fields: None, .. }
            | Definition::EnumType { variants: None, .. } => None,
            _ => {
                return Err(CompileError::Undeclared {
                    symbol_id,
                    usage_position: path_segment.position(),
                });
            }
        };

        if let Some(scope_id) = primary_member_scope_id
            && let Some(declaration_id) = self
                .resolver
                .declarations
                .find_declaration_id(symbol_id, scope_id)
        {
            return Ok(*declaration_id);
        }

        if matches!(
            parent_declaration.definition,
            Definition::StructType { .. } | Definition::EnumType { .. }
        ) && let Some(implementation_declaration_ids) =
            self.resolver.implementations.get(&parent_declaration_id)
        {
            for implementation_declaration_id in implementation_declaration_ids {
                let implementation_declaration = self
                    .resolver
                    .declarations
                    .get_declaration(*implementation_declaration_id)?;
                let implementation_scope_id = match implementation_declaration.definition {
                    Definition::InherentImplementation {
                        declarations: Some(scope_id),
                        ..
                    }
                    | Definition::TraitImplementation {
                        declarations: Some(scope_id),
                        ..
                    } => scope_id,
                    _ => continue,
                };
                if let Some(declaration_id) = self
                    .resolver
                    .declarations
                    .find_declaration_id(symbol_id, implementation_scope_id)
                {
                    return Ok(*declaration_id);
                }
            }
        }

        Err(CompileError::Undeclared {
            symbol_id,
            usage_position: path_segment.position(),
        })
    }

    fn bind_item(&mut self, reader: SyntaxReader) -> Result<(), CompileError> {
        match reader.node.kind {
            SyntaxKind::ModItem => self.bind_mod_item(reader),
            SyntaxKind::UseItem => self.bind_use_item(reader),
            SyntaxKind::FnItem => self.bind_fn_item(reader).map(|_| ()),
            SyntaxKind::StructItem => self.bind_struct_item(reader),
            SyntaxKind::EnumItem => self.bind_enum_item(reader),
            SyntaxKind::ConstItem => self.bind_const_item(reader).map(|_| ()),
            SyntaxKind::TypeItem => self.bind_type_item(reader).map(|_| ()),
            SyntaxKind::ImplItem => self.bind_impl_item(reader),
            SyntaxKind::TraitItem => self.bind_trait_item(reader),
            _ => Err(CompileError::UnexpectedSyntax {
                expected: &[
                    SyntaxKind::ModItem,
                    SyntaxKind::UseItem,
                    SyntaxKind::FnItem,
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

    fn bind_mod_item(&mut self, reader: SyntaxReader) -> Result<(), CompileError> {
        let ModItem { public, name, body } = reader.as_component()?;

        let module_name_str = self.source.get_content(name.position())?;
        let module_symbol_id = self.resolver.symbols.add_symbol(module_name_str);

        let module_declaration_id =
            self.reserve_declaration_id(module_symbol_id, Some((name.position(), reader.id)));

        self.enter_scope(ScopeKind::Module);

        let inner_scope_id = self.current_scope_id;

        if let Some(module_body) = body {
            self.resolver
                .add_declaration_binding(reader.id, module_declaration_id);

            for child in module_body.children() {
                match self.bind_item(child) {
                    Ok(()) => {}
                    Err(error) => self.errors.push(ErrorKind::Compile(error)),
                }
            }

            self.exit_scope();
            self.resolver.declarations.set_reserved_declaration(
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

            self.resolver
                .add_declaration_binding(name.id, module_declaration_id);

            let module_root = self.syntax.get_tree(module_source_id)?.root()?;

            self.bind_root(module_root)?;
            self.exit_scope();
            self.resolver.declarations.set_reserved_declaration(
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

    fn bind_use_item(&mut self, reader: SyntaxReader) -> Result<(), CompileError> {
        let UseItem { public, path } = reader.as_component()?;

        let file = self.source.get_code(path.source_id())?;
        let mut path_segments = path.children();

        let first_segment = path_segments.next().ok_or(CompileError::ExpectedSyntax {
            expected: &[SyntaxKind::PathSegment],
        })?;
        let first_segment_str = file.get_str(first_segment.node.span)?;
        let first_symbol_id = self.resolver.symbols.add_symbol(first_segment_str);
        let Some(first_declaration_id) =
            self.find_visible_declaration(first_symbol_id, first_segment)?
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
            let segment_symbol_id = self.resolver.symbols.add_symbol(segment_str);
            let segment_declaration_id =
                self.find_member_declaration(segment_symbol_id, current_declaration_id, segment)?;

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

        self.resolver
            .add_declaration_binding(reader.id, use_declaration_id);

        Ok(())
    }

    fn bind_fn_item(&mut self, reader: SyntaxReader) -> Result<DeclarationId, CompileError> {
        let FnItem {
            public,
            name,
            type_parameters,
            value_parameters,
            return_type,
            where_clause: _,
            body,
        } = reader.as_component()?;

        let function_name_str = self.source.get_content(name.position())?;
        let function_symbol_id = self.resolver.symbols.add_symbol(function_name_str);
        let function_declaration_id =
            self.reserve_declaration_id(function_symbol_id, Some((reader.position(), reader.id)));

        self.enter_scope(ScopeKind::Item);

        let type_parameters_scope_id = if let Some(type_parameters) = type_parameters {
            self.bind_type_parameters(type_parameters)?
        } else {
            None
        };
        let value_parameters_scope_id = if let Some(value_parameters) = value_parameters {
            let ValueParameters { name_type_pairs } = value_parameters.as_component()?;

            self.enter_scope(ScopeKind::Members);

            for (parameter_name, parameter_type) in name_type_pairs {
                let parameter_name_str = self.source.get_content(parameter_name.position())?;
                let parameter_symbol_id = self.resolver.symbols.add_symbol(parameter_name_str);
                let parameter_type_id = self.handle_explicit_type(parameter_type)?;
                let parameter_declaration_id = self.add_declaration(
                    parameter_symbol_id,
                    Definition::Local {
                        mutable: false,
                        shadowed: None,
                        type_id: parameter_type_id,
                    },
                    Some((parameter_name.position(), parameter_name.id)),
                );

                self.resolver
                    .add_declaration_binding(parameter_name.id, parameter_declaration_id);
            }

            Some(self.current_scope_id)
        } else {
            None
        };
        let return_type_id = if let Some(return_type) = return_type {
            self.handle_explicit_type(return_type)?
        } else {
            TypeId::UNIT
        };

        self.enter_scope(ScopeKind::Block);

        if let Some(body) = body {
            for child in body.children() {
                if child.node.kind.is_statement() {
                    match self.bind_statement(child) {
                        Ok(_) => {}
                        Err(error) => self.errors.push(ErrorKind::Compile(error)),
                    }
                } else {
                    self.bind_expression(child)?;
                }
            }
        }

        self.exit_scope();

        if value_parameters_scope_id.is_some() {
            self.exit_scope();
        }

        self.exit_scope();
        self.resolver.declarations.set_reserved_declaration(
            function_declaration_id,
            Definition::Function {
                public,
                type_parameters: type_parameters_scope_id,
                value_parameters: value_parameters_scope_id,
                return_type_id,
            },
        );
        self.resolver
            .add_declaration_binding(name.id, function_declaration_id);

        Ok(function_declaration_id)
    }

    fn bind_struct_item(&mut self, reader: SyntaxReader) -> Result<(), CompileError> {
        let StructItem {
            public,
            name,
            type_parameters,
            fields,
            ..
        } = reader.as_component()?;

        let struct_name_str = self.source.get_content(name.position())?;
        let struct_symbol_id = self.resolver.symbols.add_symbol(struct_name_str);
        let struct_declaration_id =
            self.reserve_declaration_id(struct_symbol_id, Some((reader.position(), reader.id)));

        self.enter_scope(ScopeKind::Item);

        let type_parameters_scope_id = if let Some(type_parameters) = type_parameters {
            self.bind_type_parameters(type_parameters)?
        } else {
            None
        };
        let fields = if let Some(fields) = fields.filter(|fields| fields.child_count() > 0) {
            self.enter_scope(ScopeKind::Members);

            match fields.node.kind {
                SyntaxKind::TupleFields => {
                    let TupleFields { types } = TupleFields::from_reader(&fields)?;

                    for (index, field_type) in types.enumerate() {
                        let public = field_type.node.flags.get_flag(SyntaxFlags::PUBLIC);
                        let field_symbol_id = self.resolver.symbols.add_index_symbol(index as u32);
                        let field_type_id = self.handle_explicit_type(field_type)?;

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

                    let file = self.source.get_code(name.source_id())?;

                    for (field_name, field_type) in name_type_pairs {
                        let public = field_name.node.flags.get_flag(SyntaxFlags::PUBLIC);
                        let field_name_str = file.get_str(field_name.node.span)?;
                        let field_symbol_id = self.resolver.symbols.add_symbol(field_name_str);
                        let field_type_id = self.handle_explicit_type(field_type)?;
                        let field_declaration_id = self.add_declaration(
                            field_symbol_id,
                            Definition::Field {
                                public,
                                parent_struct: struct_declaration_id,
                                type_id: field_type_id,
                            },
                            Some((field_name.position(), field_name.id)),
                        );

                        self.resolver
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
        self.resolver.declarations.set_reserved_declaration(
            struct_declaration_id,
            Definition::StructType {
                public,
                type_parameters: type_parameters_scope_id,
                fields,
            },
        );
        self.resolver
            .add_declaration_binding(reader.id, struct_declaration_id);

        Ok(())
    }

    fn bind_enum_item(&mut self, reader: SyntaxReader) -> Result<(), CompileError> {
        let EnumItem {
            public,
            name,
            type_parameters,
            variants,
        } = reader.as_component()?;

        let enum_name_str = self.source.get_content(name.position())?;
        let enum_symbol_id = self.resolver.symbols.add_symbol(enum_name_str);
        let enum_declaration_id =
            self.reserve_declaration_id(enum_symbol_id, Some((reader.position(), reader.id)));

        self.enter_scope(ScopeKind::Item);

        let type_parameters_scope_id = if let Some(type_parameters) = type_parameters {
            self.bind_type_parameters(type_parameters)?
        } else {
            None
        };

        self.enter_scope(ScopeKind::Members);

        for (index, variant) in variants.children().enumerate() {
            self.bind_enum_variant(variant, enum_declaration_id, index as u16)?;
        }

        let variants_scope_id = Some(self.current_scope_id);

        self.exit_scope();
        self.exit_scope();
        self.resolver.declarations.set_reserved_declaration(
            enum_declaration_id,
            Definition::EnumType {
                public,
                type_parameters: type_parameters_scope_id,
                variants: variants_scope_id,
            },
        );
        self.resolver
            .add_declaration_binding(name.id, enum_declaration_id);

        Ok(())
    }

    fn bind_type_parameters(
        &mut self,
        type_parameters_reader: SyntaxReader,
    ) -> Result<Option<ScopeId>, CompileError> {
        let type_parameters_scope_id = self.current_scope_id;

        for type_parameter in type_parameters_reader.children() {
            let type_parameter_name_str = self.source.get_content(type_parameter.position())?;
            let type_parameter_symbol_id =
                self.resolver.symbols.add_symbol(type_parameter_name_str);
            let type_parameter_declaration_id = self.add_declaration(
                type_parameter_symbol_id,
                Definition::TypeParameter,
                Some((type_parameter.position(), type_parameter.id)),
            );

            self.resolver
                .add_declaration_binding(type_parameter.id, type_parameter_declaration_id);
        }

        Ok(Some(type_parameters_scope_id))
    }

    fn bind_enum_variant(
        &mut self,
        reader: SyntaxReader,
        enum_declaration_id: DeclarationId,
        discriminant: u16,
    ) -> Result<DeclarationId, CompileError> {
        let file = self.source.get_code(reader.source_id())?;

        match reader.node.kind {
            SyntaxKind::EnumUnitVariant => {
                let EnumUnitVariant { name } = reader.as_component()?;

                let variant_name_str = file.get_str(name.node.span)?;
                let variant_symbol_id = self.resolver.symbols.add_symbol(variant_name_str);
                let variant_declaration_id = self.add_declaration(
                    variant_symbol_id,
                    Definition::Variant {
                        discriminant,
                        enum_declaration_id,
                        fields: None,
                    },
                    Some((name.position(), name.id)),
                );

                self.resolver
                    .add_declaration_binding(name.id, variant_declaration_id);

                Ok(variant_declaration_id)
            }
            SyntaxKind::EnumTupleFieldsVariant => {
                let EnumItemTupleVariant { name, tuple_fields } = reader.as_component()?;
                let TupleFields { types } = tuple_fields.as_component()?;

                let variant_name_str = file.get_str(name.node.span)?;
                let variant_symbol_id = self.resolver.symbols.add_symbol(variant_name_str);
                let variant_declaration_id = self.reserve_declaration_id(
                    variant_symbol_id,
                    Some((reader.position(), reader.id)),
                );

                self.enter_scope(ScopeKind::Members);

                for (index, field_type) in types.enumerate() {
                    let symbol_id = self.resolver.symbols.add_index_symbol(index as u32);
                    let type_id = self.handle_explicit_type(field_type)?;
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
                self.resolver.declarations.set_reserved_declaration(
                    variant_declaration_id,
                    Definition::Variant {
                        discriminant,
                        enum_declaration_id,
                        fields: Some(fields_scope_id),
                    },
                );
                self.resolver
                    .add_declaration_binding(name.id, variant_declaration_id);

                Ok(variant_declaration_id)
            }
            SyntaxKind::EnumNamedFieldsVariant => {
                let EnumNamedFieldsVariant { name, named_fields } = reader.as_component()?;
                let NamedFields { name_type_pairs } = named_fields.as_component()?;

                let variant_name_str = file.get_str(name.node.span)?;
                let variant_symbol_id = self.resolver.symbols.add_symbol(variant_name_str);
                let variant_declaration_id = self
                    .reserve_declaration_id(variant_symbol_id, Some((name.position(), name.id)));

                self.enter_scope(ScopeKind::Members);

                for (field_name, field_type) in name_type_pairs {
                    let field_name_str = file.get_str(field_name.node.span)?;
                    let field_symbol_id = self.resolver.symbols.add_symbol(field_name_str);
                    let field_type_id = self.handle_explicit_type(field_type)?;
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
                self.resolver.declarations.set_reserved_declaration(
                    variant_declaration_id,
                    Definition::Variant {
                        discriminant,
                        enum_declaration_id,
                        fields: Some(fields_scope_id),
                    },
                );
                self.resolver
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

    fn bind_const_item(&mut self, reader: SyntaxReader) -> Result<DeclarationId, CompileError> {
        let ConstItem {
            public,
            name,
            type_notation,
            value,
        } = reader.as_component()?;

        if let Some(value) = value {
            self.enter_scope(ScopeKind::Constant);
            self.bind_expression(value)?;
            self.exit_scope();
        }

        let const_name_str = self.source.get_content(name.position())?;
        let const_symbol_id = self.resolver.symbols.add_symbol(const_name_str);
        let const_type_id = self.handle_explicit_type(type_notation)?;
        let const_declaration_id = self.add_declaration(
            const_symbol_id,
            Definition::Constant {
                public,
                type_id: const_type_id,
            },
            Some((reader.position(), reader.id)),
        );

        self.resolver
            .add_declaration_binding(name.id, const_declaration_id);

        Ok(const_declaration_id)
    }

    fn bind_type_item(&mut self, reader: SyntaxReader) -> Result<DeclarationId, CompileError> {
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

        self.enter_scope(ScopeKind::Item);

        let type_parameters_scope_id = if let Some(type_parameters) = type_parameters {
            self.bind_type_parameters(type_parameters)?
        } else {
            None
        };

        let aliased_type_id = self.handle_explicit_type(aliased_type)?;

        self.exit_scope();

        let type_alias_name_str = self.source.get_content(name.position())?;
        let type_alias_symbol_id = self.resolver.symbols.add_symbol(type_alias_name_str);
        let type_alias_declaration_id = self.add_declaration(
            type_alias_symbol_id,
            Definition::TypeAlias {
                public,
                type_parameters: type_parameters_scope_id,
                aliased_type_id,
            },
            Some((reader.position(), reader.id)),
        );

        self.resolver
            .add_declaration_binding(name.id, type_alias_declaration_id);

        Ok(type_alias_declaration_id)
    }

    fn bind_impl_item(&mut self, reader: SyntaxReader) -> Result<(), CompileError> {
        let ImplItem {
            type_parameters,
            trait_path,
            self_name,
            type_arguments,
            where_clause,
            body,
        } = reader.as_component()?;

        self.enter_scope(ScopeKind::Item);

        let type_parameters_scope_id = if let Some(type_parameters) = type_parameters {
            self.bind_type_parameters(type_parameters)?
        } else {
            None
        };

        self.enter_scope(ScopeKind::Members);

        let trait_declaration_id = if let Some(trait_path) = trait_path {
            Some(self.bind_path(trait_path)?)
        } else {
            None
        };
        let self_type_id = self.handle_explicit_type(self_name)?;
        let self_declaration_id = match *self.resolver.types.get_type(self_type_id)? {
            Type::Algebraic { declaration_id, .. } => declaration_id,
            _ => return Err(CompileError::ExpectedConcreteType),
        };
        let previous_self_type_id = self.current_self_type_id.replace(self_type_id);

        for child in body.children() {
            match child.node.kind {
                SyntaxKind::FnItem => {
                    self.bind_fn_item(child)?;
                }
                SyntaxKind::ConstItem => {
                    self.bind_const_item(child)?;
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
                        self.bind_type_parameters(type_parameters)?
                    } else {
                        None
                    };
                    let aliased_type_id = self.handle_explicit_type(aliased_type)?;
                    let type_name_str = self.source.get_content(name.position())?;
                    let type_symbol_id = self.resolver.symbols.add_symbol(type_name_str);
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

                    self.resolver
                        .add_declaration_binding(name.id, type_declaration_id);
                }
                _ => {
                    return Err(CompileError::UnexpectedSyntax {
                        expected: &[
                            SyntaxKind::FnItem,
                            SyntaxKind::ConstItem,
                            SyntaxKind::TypeItem,
                        ],
                        found: child.node.kind,
                    });
                }
            }
        }

        let trait_type_arguments = if let Some(type_arguments) = type_arguments {
            let mut type_argument_ids =
                TypeId::SmallVec::with_capacity(type_arguments.child_count());

            for type_argument in type_arguments.children() {
                let type_argument_id = self.handle_explicit_type(type_argument)?;

                type_argument_ids.push(type_argument_id);
            }

            self.resolver.types.add_type_members(type_argument_ids)
        } else {
            TypeMembers::default()
        };

        self.current_self_type_id = previous_self_type_id;
        let impl_scope_id = Some(self.current_scope_id);

        self.exit_scope();
        self.exit_scope();

        let impl_symbol_id = self.resolver.symbols.add_impl_symbol();
        let definition = if let Some(trait_declaration_id) = trait_declaration_id {
            Definition::TraitImplementation {
                type_parameters: type_parameters_scope_id,
                trait_declaration_id,
                trait_type_arguments,
                declarations: impl_scope_id,
            }
        } else {
            Definition::InherentImplementation {
                type_parameters: type_parameters_scope_id,
                declarations: impl_scope_id,
            }
        };
        let impl_declaration_id = self.add_declaration(
            impl_symbol_id,
            definition,
            Some((reader.position(), reader.id)),
        );

        self.resolver
            .add_declaration_binding(reader.id, impl_declaration_id);

        self.resolver
            .implementations
            .entry(self_declaration_id)
            .or_default()
            .push(impl_declaration_id);

        Ok(())
    }

    fn bind_trait_item(&mut self, reader: SyntaxReader) -> Result<(), CompileError> {
        let TraitItem {
            public,
            name,
            body,
            type_parameters,
            supertraits,
            where_clause: _,
        } = reader.as_component()?;

        let trait_name_str = self.source.get_content(name.position())?;
        let trait_symbol_id = self.resolver.symbols.add_symbol(trait_name_str);
        let trait_declaration_id =
            self.reserve_declaration_id(trait_symbol_id, Some((reader.position(), reader.id)));

        self.enter_scope(ScopeKind::Item);

        let type_parameters_scope_id = if let Some(type_parameters) = type_parameters {
            self.bind_type_parameters(type_parameters)?
        } else {
            None
        };

        let supertraits_scope_id = if let Some(supertraits) = supertraits {
            self.enter_scope(ScopeKind::Members);

            for supertrait in supertraits.children() {
                let supertrait_declaration_id = self.bind_path(supertrait)?;

                self.resolver
                    .scopes
                    .add_to_current_namespace(supertrait_declaration_id);
            }

            Some(self.current_scope_id)
        } else {
            None
        };

        self.enter_scope(ScopeKind::Members);

        let self_symbol_id = self.resolver.symbols.add_self_symbol();
        let self_type_declaration_id = self.resolver.declarations.add_declaration(Declaration {
            symbol_id: self_symbol_id,
            scope_id: self.current_scope_id,
            definition: Definition::TypeParameter,
            syntax: None,
        });
        let self_type_id = self.resolver.types.add_type(Type::Generic {
            declaration_id: self_type_declaration_id,
        });
        let previous_self_type_id = self.current_self_type_id.replace(self_type_id);

        for child in body.children() {
            match child.node.kind {
                SyntaxKind::FnItem => {
                    self.bind_fn_item(child)?;
                }
                SyntaxKind::TypeItem => {
                    let TypeItem {
                        public,
                        name,
                        type_parameters,
                        aliased_type,
                    } = child.as_component()?;

                    let type_parameters_scope_id = if let Some(type_parameters) = type_parameters {
                        self.bind_type_parameters(type_parameters)?
                    } else {
                        None
                    };
                    let aliased_type_id = aliased_type
                        .map(|aliased_type| self.handle_explicit_type(aliased_type))
                        .transpose()?;
                    let type_name_str = self.source.get_content(name.position())?;
                    let type_symbol_id = self.resolver.symbols.add_symbol(type_name_str);
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

                    self.resolver
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
                        self.bind_expression(value)?;
                    }

                    let const_name_str = self.source.get_content(name.position())?;
                    let const_symbol_id = self.resolver.symbols.add_symbol(const_name_str);
                    let const_type_id = self.handle_explicit_type(type_notation)?;
                    let const_declaration_id = self.add_declaration(
                        const_symbol_id,
                        Definition::InherentAssociatedConstant {
                            public: false,
                            parent: trait_declaration_id,
                            type_id: const_type_id,
                        },
                        Some((child.position(), child.id)),
                    );

                    self.resolver
                        .add_declaration_binding(name.id, const_declaration_id);
                }
                _ => {
                    return Err(CompileError::UnexpectedSyntax {
                        expected: &[
                            SyntaxKind::FnItem,
                            SyntaxKind::TypeItem,
                            SyntaxKind::ConstItem,
                        ],
                        found: child.node.kind,
                    });
                }
            };
        }

        self.current_self_type_id = previous_self_type_id;
        let declarations_scope_id = Some(self.current_scope_id);

        self.exit_scope();

        if supertraits_scope_id.is_some() {
            self.exit_scope();
        }

        self.exit_scope();
        self.resolver.declarations.set_reserved_declaration(
            trait_declaration_id,
            Definition::Trait {
                public,
                type_parameters: type_parameters_scope_id,
                supertraits: supertraits_scope_id,
                declarations: declarations_scope_id,
            },
        );
        self.resolver
            .add_declaration_binding(name.id, trait_declaration_id);

        Ok(())
    }

    fn bind_statement(&mut self, reader: SyntaxReader) -> Result<(), CompileError> {
        match reader.node.kind {
            SyntaxKind::UseItem => self.bind_use_item(reader),
            SyntaxKind::FnItem => self.bind_fn_item(reader).map(|_| ()),
            SyntaxKind::TypeItem => self.bind_type_item(reader).map(|_| ()),
            SyntaxKind::ConstItem => self.bind_const_item(reader).map(|_| ()),
            SyntaxKind::StructItem => self.bind_struct_item(reader),
            SyntaxKind::EnumItem => self.bind_enum_item(reader),
            SyntaxKind::ImplItem => self.bind_impl_item(reader),
            SyntaxKind::TraitItem => self.bind_trait_item(reader),
            SyntaxKind::LetStatement => self.bind_let_statement(reader),
            SyntaxKind::ExpressionStatement => self.bind_expression_statement(reader),
            _ => Err(CompileError::UnexpectedSyntax {
                expected: &[
                    SyntaxKind::ConstItem,
                    SyntaxKind::EnumItem,
                    SyntaxKind::ExpressionStatement,
                    SyntaxKind::FnItem,
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

    fn bind_let_statement(&mut self, reader: SyntaxReader) -> Result<(), CompileError> {
        let LetStatement {
            mutable,
            name,
            expression,
            type_notation,
        } = reader.as_component()?;

        self.bind_expression(expression)?;

        let local_name_str = self.source.get_content(name.position())?;
        let local_symbol_id = self.resolver.symbols.add_symbol(local_name_str);
        let local_type_id = if let Some(type_notation) = type_notation {
            self.handle_explicit_type(type_notation)?
        } else {
            self.resolver.types.create_inferred_type(None)
        };
        let shadowed = self
            .resolver
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

        self.resolver
            .add_declaration_binding(name.id, declaration_id);

        Ok(())
    }

    fn bind_expression_statement(&mut self, reader: SyntaxReader) -> Result<(), CompileError> {
        let ExpressionStatement { expression } = reader.as_component()?;

        self.bind_expression(expression)?;

        Ok(())
    }

    fn bind_expression(&mut self, reader: SyntaxReader) -> Result<(), CompileError> {
        match reader.node.kind {
            SyntaxKind::BooleanExpression => self.bind_boolean_expression(reader),
            SyntaxKind::HexadecimalExpression => self.bind_hexadecimal_expression(reader),
            SyntaxKind::CharacterExpression => self.bind_character_expression(reader),
            SyntaxKind::FloatExpression => self.bind_float_expression(reader),
            SyntaxKind::IntegerExpression => self.bind_integer_expression(reader),
            SyntaxKind::StringExpression => self.bind_string_expression(reader),
            SyntaxKind::ArrayExpression => self.bind_array_expression(reader),
            SyntaxKind::ArrayRepeatExpression => self.bind_array_repeat_expression(reader),
            SyntaxKind::IndexExpression => self.bind_index_expression(reader),
            SyntaxKind::RangeExpression => self.bind_range_expression(reader),
            SyntaxKind::PathExpression => self.bind_path_expression(reader),
            SyntaxKind::StructExpression => self.bind_struct_expression(reader),
            SyntaxKind::GroupedExpression => self.bind_grouped_expression(reader),
            SyntaxKind::BlockExpression => self.bind_block_expression(reader),
            SyntaxKind::IfExpression => self.bind_if_expression(reader),
            SyntaxKind::NegationExpression => self.bind_negation_expression(reader),
            SyntaxKind::NotExpression => self.bind_not_expression(reader),
            SyntaxKind::WhileExpression => self.bind_while_expression(reader),
            SyntaxKind::BreakExpression => self.bind_break_expression(reader),
            SyntaxKind::CallExpression => self.bind_call_expression(reader),
            SyntaxKind::FieldAccessExpression => self.bind_field_access_expression(reader),
            SyntaxKind::AndExpression | SyntaxKind::OrExpression => {
                self.bind_logic_expression(reader)
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
            | SyntaxKind::ExponentAssignmentExpression => self.bind_math_expression(reader),
            SyntaxKind::AssignmentExpression => self.bind_assignment_expression(reader),
            SyntaxKind::GreaterThanExpression
            | SyntaxKind::LessThanExpression
            | SyntaxKind::GreaterThanOrEqualExpression
            | SyntaxKind::LessThanOrEqualExpression
            | SyntaxKind::EqualExpression
            | SyntaxKind::NotEqualExpression => self.bind_comparison_expression(reader),
            _ => Err(CompileError::UnexpectedSyntax {
                expected: &[
                    SyntaxKind::AdditionExpression,
                    SyntaxKind::SubtractionExpression,
                    SyntaxKind::MultiplicationExpression,
                    SyntaxKind::DivisionExpression,
                    SyntaxKind::ModuloExpression,
                    SyntaxKind::ExponentExpression,
                    SyntaxKind::AssignmentExpression,
                    SyntaxKind::AndExpression,
                    SyntaxKind::OrExpression,
                    SyntaxKind::GreaterThanExpression,
                    SyntaxKind::LessThanExpression,
                    SyntaxKind::GreaterThanOrEqualExpression,
                    SyntaxKind::LessThanOrEqualExpression,
                    SyntaxKind::EqualExpression,
                    SyntaxKind::NotEqualExpression,
                    SyntaxKind::BooleanExpression,
                    SyntaxKind::HexadecimalExpression,
                    SyntaxKind::CharacterExpression,
                    SyntaxKind::FloatExpression,
                    SyntaxKind::IntegerExpression,
                    SyntaxKind::StringExpression,
                    SyntaxKind::ArrayExpression,
                    SyntaxKind::ArrayRepeatExpression,
                    SyntaxKind::IndexExpression,
                    SyntaxKind::RangeExpression,
                    SyntaxKind::PathExpression,
                    SyntaxKind::StructExpression,
                    SyntaxKind::GroupedExpression,
                    SyntaxKind::BlockExpression,
                    SyntaxKind::IfExpression,
                    SyntaxKind::NegationExpression,
                    SyntaxKind::NotExpression,
                    SyntaxKind::WhileExpression,
                    SyntaxKind::BreakExpression,
                    SyntaxKind::CallExpression,
                    SyntaxKind::FieldAccessExpression,
                ],
                found: reader.node.kind,
            }),
        }
    }

    fn bind_assignment_expression(&mut self, reader: SyntaxReader) -> Result<(), CompileError> {
        let AssignmentExpression { target, source } = reader.as_component()?;

        self.bind_expression(target)?;
        self.bind_expression(source)?;

        Ok(())
    }

    fn bind_boolean_expression(&mut self, _: SyntaxReader) -> Result<(), CompileError> {
        Ok(())
    }

    fn bind_hexadecimal_expression(&mut self, _: SyntaxReader) -> Result<(), CompileError> {
        Ok(())
    }

    fn bind_character_expression(&mut self, _: SyntaxReader) -> Result<(), CompileError> {
        Ok(())
    }

    fn bind_float_expression(&mut self, _: SyntaxReader) -> Result<(), CompileError> {
        Ok(())
    }

    fn bind_integer_expression(&mut self, _: SyntaxReader) -> Result<(), CompileError> {
        Ok(())
    }

    fn bind_string_expression(&mut self, _: SyntaxReader) -> Result<(), CompileError> {
        Ok(())
    }

    fn bind_array_expression(&mut self, reader: SyntaxReader) -> Result<(), CompileError> {
        let ArrayExpression { elements } = reader.as_component()?;

        for element in elements {
            self.bind_expression(element)?;
        }

        Ok(())
    }

    fn bind_array_repeat_expression(&mut self, reader: SyntaxReader) -> Result<(), CompileError> {
        let ArrayRepeatExpression { element, .. } = reader.as_component()?;

        self.bind_expression(element)?;

        Ok(())
    }

    fn bind_index_expression(&mut self, reader: SyntaxReader) -> Result<(), CompileError> {
        let IndexExpression { collection, index } = reader.as_component()?;

        self.bind_expression(collection)?;
        self.bind_expression(index)?;

        Ok(())
    }

    fn bind_range_expression(&mut self, reader: SyntaxReader) -> Result<(), CompileError> {
        let RangeExpression { start, end } = reader.as_component()?;

        self.bind_expression(start)?;
        self.bind_expression(end)?;

        Ok(())
    }

    fn bind_path_expression(&mut self, reader: SyntaxReader) -> Result<(), CompileError> {
        if let Some(declaration_id) = self.bind_path_segments(reader)? {
            self.resolver
                .add_declaration_binding(reader.id, declaration_id);
        }

        Ok(())
    }

    fn bind_struct_expression(&mut self, reader: SyntaxReader) -> Result<(), CompileError> {
        let StructExpression { path, fields } = reader.as_component()?;
        let StructExpressionStructFields {
            name_expression_pairs,
        } = fields.as_component()?;

        let struct_declaration_id = self.bind_path(path)?;
        let struct_declaration = self
            .resolver
            .declarations
            .get_declaration(struct_declaration_id)?;
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
            let field_symbol_id = self.resolver.symbols.add_symbol(field_name_str);

            if let Some(fields_scope_id) = fields_scope_id
                && let Some(field_declaration_id) = self
                    .resolver
                    .declarations
                    .find_declaration_id(field_symbol_id, fields_scope_id)
            {
                self.resolver
                    .add_declaration_binding(field_name.id, *field_declaration_id);
            }

            self.bind_expression(field_value)?;
        }

        Ok(())
    }

    fn bind_grouped_expression(&mut self, reader: SyntaxReader) -> Result<(), CompileError> {
        let GroupedExpression { expression } = reader.as_component()?;

        if let Some(expression) = expression {
            self.bind_expression(expression)
        } else {
            Ok(())
        }
    }

    fn bind_block_expression(&mut self, reader: SyntaxReader) -> Result<(), CompileError> {
        let BlockExpression { children } = reader.as_component()?;

        self.enter_scope(ScopeKind::Block);

        for child in children {
            if child.node.kind.is_statement() {
                match self.bind_statement(child) {
                    Ok(_) => {}
                    Err(error) => self.errors.push(ErrorKind::Compile(error)),
                }
            } else {
                self.bind_expression(child)?;
            }
        }

        self.exit_scope();

        Ok(())
    }

    fn bind_if_expression(&mut self, reader: SyntaxReader) -> Result<(), CompileError> {
        let IfExpression {
            condition,
            then_branch,
            else_branch,
        } = reader.as_component()?;

        self.bind_expression(condition)?;
        self.bind_block_expression(then_branch)?;

        if let Some(else_branch) = else_branch {
            match else_branch.node.kind {
                SyntaxKind::BlockExpression => self.bind_block_expression(else_branch)?,
                SyntaxKind::IfExpression => self.bind_if_expression(else_branch)?,
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

    fn bind_math_expression(&mut self, reader: SyntaxReader) -> Result<(), CompileError> {
        let MathExpression { left, right } = reader.as_component()?;

        self.bind_expression(left)?;
        self.bind_expression(right)?;

        Ok(())
    }

    fn bind_comparison_expression(&mut self, reader: SyntaxReader) -> Result<(), CompileError> {
        let ComparisonExpression { left, right } = reader.as_component()?;

        self.bind_expression(left)?;
        self.bind_expression(right)?;

        Ok(())
    }

    fn bind_logic_expression(&mut self, reader: SyntaxReader) -> Result<(), CompileError> {
        let LogicExpression { left, right } = reader.as_component()?;

        self.bind_expression(left)?;
        self.bind_expression(right)?;

        Ok(())
    }

    fn bind_negation_expression(&mut self, reader: SyntaxReader) -> Result<(), CompileError> {
        let NegationExpression { operand } = reader.as_component()?;

        self.bind_expression(operand)?;

        Ok(())
    }

    fn bind_not_expression(&mut self, reader: SyntaxReader) -> Result<(), CompileError> {
        let NotExpression { operand } = reader.as_component()?;

        self.bind_expression(operand)?;

        Ok(())
    }

    fn bind_while_expression(&mut self, reader: SyntaxReader) -> Result<(), CompileError> {
        let WhileExpression { condition, body } = reader.as_component()?;

        self.bind_expression(condition)?;
        self.bind_block_expression(body)?;

        Ok(())
    }

    fn bind_break_expression(&mut self, _: SyntaxReader) -> Result<(), CompileError> {
        Ok(())
    }

    fn bind_call_expression(&mut self, reader: SyntaxReader) -> Result<(), CompileError> {
        let CallExpression { callee, arguments } = reader.as_component()?;

        self.bind_expression(callee)?;

        let callee_declaration_id = *self.resolver.get_declaration_binding(&callee.id)?;

        self.resolver
            .add_declaration_binding(reader.id, callee_declaration_id);

        for argument in arguments.children() {
            self.bind_expression(argument)?;
        }

        Ok(())
    }

    fn bind_field_access_expression(&mut self, reader: SyntaxReader) -> Result<(), CompileError> {
        let FieldAccessExpression {
            struct_expression,
            field_name,
        } = reader.as_component()?;

        self.bind_expression(struct_expression)?;

        let struct_expression_declaration_id = *self
            .resolver
            .get_declaration_binding(&struct_expression.id)?;
        let struct_expression_declaration = self
            .resolver
            .declarations
            .get_declaration(struct_expression_declaration_id)?;
        let Definition::Local { type_id, .. } = struct_expression_declaration.definition else {
            return Err(CompileError::ExpectedLocalDefinition(
                struct_expression_declaration_id,
            ));
        };
        let Type::Algebraic {
            declaration_id: struct_declaration_id,
            ..
        } = self.resolver.types.get_type(type_id)?
        else {
            return Err(CompileError::ExpectedStructDefinition(
                struct_expression_declaration_id,
            ));
        };
        let struct_declaration = self
            .resolver
            .declarations
            .get_declaration(*struct_declaration_id)?;
        let Definition::StructType {
            fields: Some(fields_scope_id),
            ..
        } = struct_declaration.definition
        else {
            return Err(CompileError::ExpectedStructDefinition(
                *struct_declaration_id,
            ));
        };
        let field_name_str = self.source.get_content(field_name.position())?;
        let field_symbol_id = self.resolver.symbols.add_symbol(field_name_str);
        let field_declaration_id = self
            .resolver
            .declarations
            .find_declaration_id(field_symbol_id, fields_scope_id)
            .copied()
            .ok_or(CompileError::Undeclared {
                symbol_id: field_symbol_id,
                usage_position: field_name.position(),
            })?;

        self.resolver
            .add_declaration_binding(field_name.id, field_declaration_id);

        Ok(())
    }

    fn handle_explicit_type(&mut self, reader: SyntaxReader) -> Result<TypeId, CompileError> {
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

                let element_type_id = self.handle_explicit_type(element_type)?;
                let length_str = self.source.get_content(length.position())?;
                let length = create_usize_from_decimal(length_str)?;

                Ok(self.resolver.types.add_type(Type::Array {
                    element_type_id,
                    length,
                }))
            }
            SyntaxKind::TupleType => {
                let TupleType { element_types } = reader.as_component()?;

                let element_type_ids = element_types
                    .map(|element_type| self.handle_explicit_type(element_type))
                    .try_collect::<TypeId::SmallVec>()?;
                let element_types = self.resolver.types.add_type_members(element_type_ids);

                Ok(self.resolver.types.add_type(Type::Tuple { element_types }))
            }
            SyntaxKind::FunctionType => {
                let FunctionType {
                    value_parameter_types,
                    return_type,
                } = reader.as_component()?;

                let value_parameter_ids = value_parameter_types
                    .children()
                    .map(|parameter_type| self.handle_explicit_type(parameter_type))
                    .try_collect::<TypeId::SmallVec>()?;
                let value_parameters = self.resolver.types.add_type_members(value_parameter_ids);
                let return_type_id = if let Some(return_type) = return_type {
                    self.handle_explicit_type(return_type)?
                } else {
                    TypeId::UNIT
                };

                Ok(self.resolver.types.add_type(Type::Function {
                    value_parameters,
                    return_type_id,
                }))
            }
            SyntaxKind::TypePath => {
                if let Some(declaration_id) = self.bind_path_segments(reader)? {
                    self.resolver
                        .add_declaration_binding(reader.id, declaration_id);

                    let declaration = self.resolver.declarations.get_declaration(declaration_id)?;

                    match declaration.definition {
                        Definition::StructType { .. } | Definition::EnumType { .. } => {
                            Ok(self.resolver.types.add_type(Type::Algebraic {
                                declaration_id,
                                type_arguments: TypeMembers::default(),
                            }))
                        }
                        Definition::TypeParameter => Ok(self
                            .resolver
                            .types
                            .add_type(Type::Generic { declaration_id })),
                        _ => Err(CompileError::ExpectedTypeDeclaration(declaration_id)),
                    }
                } else {
                    Ok(self.resolver.types.create_inferred_type(None))
                }
            }
            SyntaxKind::SelfType => self.current_self_type_id.ok_or_else(|| {
                let symbol_id = self.resolver.symbols.add_symbol("Self");

                CompileError::Undeclared {
                    symbol_id,
                    usage_position: reader.position(),
                }
            }),
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

    fn bind_path(&mut self, path: SyntaxReader) -> Result<DeclarationId, CompileError> {
        debug_assert_eq!(path.node.kind, SyntaxKind::Path);

        self.bind_path_segments(path)?
            .ok_or_else(|| CompileError::ExpectedValue {
                source_id: path.source_id(),
                syntax_id: path.id,
            })
    }
}
