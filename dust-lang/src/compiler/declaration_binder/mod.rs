#[cfg(test)]
mod tests;

use smallvec::{SmallVec, smallvec};
use tracing::debug;

use crate::{
    compiler::{error::CompileError, value_creation::create_usize_from_decimal},
    error::ErrorKind,
    resolver::{
        Resolver,
        declarations::{
            Declaration, DeclarationId, DeclarationMembers, Definition, ModuleKind, Visibility,
        },
        scopes::{Scope, ScopeId, ScopeKind},
        symbols::SymbolId,
        types::{Type, TypeId, TypeMembers},
    },
    source::{Position, Source, Span},
    syntax::{
        Syntax,
        components::{
            ArrayExpression, ArrayRepeatExpression, ArrayType, AssignmentExpression,
            CallExpression, ComparisonExpression, CompoundAssignmentExpression, ConstItem,
            EnumItem, EnumVariant, ExpressionStatement, FieldAccessExpression, FunctionItem,
            FunctionParameters, FunctionType, GroupedExpression, IfExpression, ImplItem,
            ImplTraitItem, IndexExpression, LetStatement, LogicExpression, MathExpression,
            ModuleItem, NegationExpression, NotExpression, RangeExpression, StructExpression,
            StructExpressionStructFields, StructItem, StructItemStructFields,
            StructItemTupleFields, SyntaxComponent, TraitBounds, TraitConst, TraitItem,
            TraitMethod, TraitType, TypeItem, UseItem, WhileExpression,
        },
        node::SyntaxKind,
        reader::SyntaxReader,
        visitor::SyntaxVisitor,
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
            current_scope_id: starting_scope_id,
            current_self_type_id: None,
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

    fn visit_root(&mut self, reader: SyntaxReader) -> Result<Self::RootOutput, CompileError> {
        debug_assert_eq!(reader.node.kind, SyntaxKind::Root);

        for item in reader.children() {
            match self.visit_item(item) {
                Ok(()) => {}
                Err(error) => self.errors.push(ErrorKind::Compile(error)),
            }
        }

        Ok(())
    }

    fn visit_module_item(&mut self, reader: SyntaxReader) -> Result<(), CompileError> {
        let ModuleItem { public, name, body } = reader.as_component()?;

        let module_name_str = self
            .source
            .get_file(name.file_id())?
            .content_str(name.node.span)?;
        let module_symbol_id = self.resolver.symbols.add_symbol(module_name_str);
        let module_scope_id = self.resolver.scopes.add_scope(Scope {
            kind: ScopeKind::Module,
            parent: self.current_scope_id,
            modules: smallvec![ScopeId::CORE],
            imports: SmallVec::new(),
        });

        if let Some(module_body) = body {
            let module_declaration_id = self.resolver.declarations.add_declaration(Declaration {
                symbol_id: module_symbol_id,
                definition: Definition::Module {
                    public,
                    kind: ModuleKind::Inline,
                    inner_scope_id: module_scope_id,
                },
                scope_id: self.current_scope_id,
                syntax: Some((name.position(), reader.id)),
            });

            self.resolver
                .add_scope_binding(module_body.id, module_scope_id);
            self.resolver
                .add_declaration_binding(reader.id, module_declaration_id);

            let starting_scope_id = self.current_scope_id;
            self.current_scope_id = module_scope_id;

            for child in module_body.children() {
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
                definition: Definition::Module {
                    public,
                    kind: ModuleKind::File {
                        file_id: module_file_id,
                    },
                    inner_scope_id: module_scope_id,
                },
                scope_id: self.current_scope_id,
                syntax: Some((name.position(), reader.id)),
            });

            self.resolver
                .add_declaration_binding(name.id, module_declaration_id);

            let module_root = self.syntax.get_tree(module_file_id)?.root()?;
            let starting_scope_id = self.current_scope_id;
            self.current_scope_id = module_scope_id;

            self.visit_root(module_root)?;

            self.current_scope_id = starting_scope_id;
        }

        Ok(())
    }

    fn visit_use_item(&mut self, reader: SyntaxReader) -> Result<(), CompileError> {
        let start = reader.node.span.start();
        let UseItem { public, path } = reader.as_component()?;

        let file = self.source.get_file(path.file_id())?;

        let mut current_scope_id = self.current_scope_id;
        let mut current_declaration_id = None;
        let mut current_end = start;
        let mut symbol_id = self
            .resolver
            .symbols
            .add_symbol(file.content_str(path.node.span)?);

        let mut path_segments = path.children();

        'outer: while let Some(segment) = path_segments.next() {
            let segment_str = file.content_str(segment.node.span)?;
            let segment_symbol_id = self.resolver.symbols.add_symbol(segment_str);
            let (declaration_id, declaration) = self.resolver.find_declaration_in_scope(
                segment_symbol_id,
                current_scope_id,
                Visibility::Module,
                &segment,
            )?;

            current_declaration_id = Some(declaration_id);
            current_end = segment.node.span.end();
            symbol_id = segment_symbol_id;

            match declaration.definition {
                Definition::Module { inner_scope_id, .. } => {
                    current_scope_id = inner_scope_id;
                }
                Definition::EnumType {
                    public, variants, ..
                } => {
                    if !public {
                        return Err(CompileError::CannotImport {
                            declaration_id,
                            position: segment.position(),
                        });
                    }

                    if let Some(next_segment) = path_segments.next() {
                        let segment_str = file.content_str(next_segment.node.span)?;
                        let segment_symbol_id = self.resolver.symbols.add_symbol(segment_str);
                        let variant_ids = self
                            .resolver
                            .declarations
                            .get_declaration_members(&variants)?;

                        for variant_id in variant_ids {
                            let variant_declaration =
                                self.resolver.declarations.get_declaration(*variant_id)?;

                            if variant_declaration.symbol_id == segment_symbol_id
                                && let Definition::Variant { parent_enum, .. } =
                                    variant_declaration.definition
                                && parent_enum == declaration_id
                            {
                                current_declaration_id = Some(*variant_id);
                                current_end = next_segment.node.span.end();
                                symbol_id = segment_symbol_id;

                                break 'outer;
                            }
                        }
                    } else {
                        break;
                    }
                }
                _ => {
                    break;
                }
            }
        }

        if let Some(current_declaration_id) = current_declaration_id {
            let use_declaration_id = self.resolver.declarations.add_declaration(Declaration {
                symbol_id,
                definition: Definition::Use {
                    public,
                    item: current_declaration_id,
                },
                scope_id: self.current_scope_id,
                syntax: Some((
                    Position::new(reader.file_id(), Span::new(start, current_end)),
                    reader.id,
                )),
            });

            self.resolver
                .add_declaration_binding(reader.id, use_declaration_id);
        }

        Ok(())
    }

    fn visit_function_item(&mut self, reader: SyntaxReader) -> Result<(), CompileError> {
        let FunctionItem {
            public,
            name,
            parameters,
            return_type,
            body,
            ..
        } = reader.as_component()?;
        let FunctionParameters {
            value_parameters,
            type_parameters,
        } = parameters.as_component()?;

        let starting_scope_id = self.current_scope_id;
        self.current_scope_id = self.resolver.scopes.add_scope(Scope {
            kind: ScopeKind::Function,
            parent: self.current_scope_id,
            modules: smallvec![ScopeId::CORE],
            imports: SmallVec::new(),
        });

        let mut type_parameter_declaration_ids = SmallVec::<[DeclarationId; 4]>::new();

        if let Some(type_parameters) = type_parameters {
            type_parameter_declaration_ids.reserve(type_parameters.child_count());

            for type_parameter in type_parameters.children() {
                let type_parameter_name_str =
                    self.source.get_file_content(&type_parameter.position())?;
                let type_parameter_symbol_id =
                    self.resolver.symbols.add_symbol(type_parameter_name_str);
                let type_parameter_declaration_id =
                    self.resolver.declarations.add_declaration(Declaration {
                        symbol_id: type_parameter_symbol_id,
                        definition: Definition::TypeParameter,
                        scope_id: self.current_scope_id,
                        syntax: Some((type_parameter.position(), type_parameter.id)),
                    });

                type_parameter_declaration_ids.push(type_parameter_declaration_id);
                self.resolver
                    .add_declaration_binding(type_parameter.id, type_parameter_declaration_id);
            }
        }

        let value_parameters = {
            let mut type_ids =
                SmallVec::<[TypeId; 4]>::with_capacity(value_parameters.child_count());

            for [parameter_name, parameter_type] in value_parameters.children().array_chunks() {
                let parameter_name_str =
                    self.source.get_file_content(&parameter_name.position())?;
                let parameter_symbol_id = self.resolver.symbols.add_symbol(parameter_name_str);
                let parameter_type_id = self.visit_type(parameter_type)?;
                let parameter_declaration_id =
                    self.resolver.declarations.add_declaration(Declaration {
                        symbol_id: parameter_symbol_id,
                        definition: Definition::Local {
                            mutable: false,
                            shadowed: None,
                            type_id: parameter_type_id,
                        },
                        scope_id: self.current_scope_id,
                        syntax: Some((parameter_name.position(), parameter_name.id)),
                    });

                self.resolver
                    .add_declaration_binding(parameter_name.id, parameter_declaration_id);

                if let Ok(Type::Slice { declaration_id, .. }) =
                    self.resolver.types.get_type(parameter_type_id)
                {
                    type_parameter_declaration_ids.push(*declaration_id);
                }

                type_ids.push(parameter_type_id);
            }

            self.resolver.types.add_type_members(type_ids)
        };

        let type_parameters = self
            .resolver
            .declarations
            .add_declaration_members(type_parameter_declaration_ids);
        let return_type_id = if let Some(return_type) = return_type {
            self.visit_type(return_type)?
        } else {
            TypeId::UNIT
        };
        let function_name_str = self.source.get_file_content(&name.position())?;
        let function_symbol_id = self.resolver.symbols.add_symbol(function_name_str);
        let function_declaration_id = self.resolver.declarations.add_declaration(Declaration {
            symbol_id: function_symbol_id,
            definition: Definition::Function {
                public,
                type_parameters,
                value_parameters,
                return_type_id,
            },
            scope_id: starting_scope_id,
            syntax: Some((reader.position(), reader.id)),
        });

        self.resolver
            .add_declaration_binding(name.id, function_declaration_id);
        self.resolver
            .add_scope_binding(body.id, self.current_scope_id);

        for child in body.children() {
            if child.node.kind.is_statement() {
                match self.visit_statement(child) {
                    Ok(_) => {}
                    Err(error) => self.errors.push(ErrorKind::Compile(error)),
                }
            } else {
                self.visit_expression(child, None)?;
            }
        }

        self.current_scope_id = starting_scope_id;

        Ok(())
    }

    fn visit_struct_item(&mut self, reader: SyntaxReader) -> Result<(), CompileError> {
        let StructItem {
            public,
            name,
            type_parameters,
            fields,
            ..
        } = reader.as_component()?;

        let struct_name_str = self.source.get_file_content(&name.position())?;
        let struct_symbol = self.resolver.symbols.add_symbol(struct_name_str);
        let struct_declaration_id = self.resolver.declarations.reserve_declaration_id();

        let mut type_parameter_declaration_ids = SmallVec::<[DeclarationId; 4]>::new();

        if let Some(type_parameters) = type_parameters {
            type_parameter_declaration_ids.reserve(type_parameters.child_count());

            for type_parameter in type_parameters.children() {
                let type_parameter_name_str =
                    self.source.get_file_content(&type_parameter.position())?;
                let type_parameter_symbol_id =
                    self.resolver.symbols.add_symbol(type_parameter_name_str);
                let type_parameter_declaration_id =
                    self.resolver.declarations.add_declaration(Declaration {
                        symbol_id: type_parameter_symbol_id,
                        definition: Definition::TypeParameter,
                        scope_id: self.current_scope_id,
                        syntax: Some((type_parameter.position(), type_parameter.id)),
                    });

                type_parameter_declaration_ids.push(type_parameter_declaration_id);
                self.resolver
                    .add_declaration_binding(type_parameter.id, type_parameter_declaration_id);
            }
        }

        let type_parameters = self
            .resolver
            .declarations
            .add_declaration_members(type_parameter_declaration_ids);

        let mut field_declaration_ids = SmallVec::<[DeclarationId; 4]>::new();

        match fields.node.kind {
            SyntaxKind::StructItemTupleFields => {
                let StructItemTupleFields { types } = StructItemTupleFields::from_reader(&fields)?;

                for (index, field_type) in types.enumerate() {
                    let public = field_type.node.modifier;
                    let symbol_id = self.resolver.symbols.add_index_symbol(index as u32);
                    let type_id = self.visit_type(field_type)?;
                    let field_declaration_id =
                        self.resolver.declarations.add_declaration(Declaration {
                            symbol_id,
                            definition: Definition::Field {
                                public,
                                parent_struct: struct_declaration_id,
                                type_id,
                            },
                            scope_id: self.current_scope_id,
                            syntax: Some((field_type.position(), field_type.id)),
                        });

                    self.resolver
                        .add_declaration_binding(field_type.id, field_declaration_id);
                    field_declaration_ids.push(field_declaration_id);
                }
            }
            SyntaxKind::StructItemStructFields => {
                let StructItemStructFields { name_type_pairs } =
                    StructItemStructFields::from_reader(&fields)?;

                let file = self.source.get_file(name.file_id())?;

                for [field_name, field_type] in name_type_pairs {
                    let public = field_name.node.modifier;
                    let field_name_str = file.content_str(field_name.node.span)?;
                    let field_symbol_id = self.resolver.symbols.add_symbol(field_name_str);
                    let field_type_id = self.visit_type(field_type)?;
                    let field_declaration_id =
                        self.resolver.declarations.add_declaration(Declaration {
                            symbol_id: field_symbol_id,
                            definition: Definition::Field {
                                public,
                                parent_struct: struct_declaration_id,
                                type_id: field_type_id,
                            },
                            scope_id: self.current_scope_id,
                            syntax: Some((field_name.position(), field_name.id)),
                        });

                    self.resolver
                        .add_declaration_binding(field_name.id, field_declaration_id);
                    field_declaration_ids.push(field_declaration_id);
                }
            }
            SyntaxKind::StructItemUnit => {}
            _ => {
                return Err(CompileError::ExpectedSyntaxKinds {
                    expected: &[
                        SyntaxKind::StructItemTupleFields,
                        SyntaxKind::StructItemStructFields,
                        SyntaxKind::StructItemUnit,
                    ],
                    found: fields.node.kind,
                });
            }
        }

        let fields = self
            .resolver
            .declarations
            .add_declaration_members(field_declaration_ids);

        self.resolver.declarations.set_declaration(
            struct_declaration_id,
            Declaration {
                symbol_id: struct_symbol,
                definition: Definition::StructType {
                    public,
                    type_parameters,
                    fields,
                },
                scope_id: self.current_scope_id,
                syntax: Some((reader.position(), reader.id)),
            },
        );

        self.resolver
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

        let enum_name_str = self.source.get_file_content(&name.position())?;
        let enum_symbol = self.resolver.symbols.add_symbol(enum_name_str);
        let enum_declaration_id = self.resolver.declarations.reserve_declaration_id();

        let mut type_parameter_declaration_ids = SmallVec::<[DeclarationId; 4]>::new();

        if let Some(type_parameters) = type_parameters {
            type_parameter_declaration_ids.reserve(type_parameters.child_count());

            for type_parameter in type_parameters.children() {
                let type_parameter_name_str =
                    self.source.get_file_content(&type_parameter.position())?;
                let type_parameter_symbol_id =
                    self.resolver.symbols.add_symbol(type_parameter_name_str);
                let type_parameter_declaration_id =
                    self.resolver.declarations.add_declaration(Declaration {
                        symbol_id: type_parameter_symbol_id,
                        definition: Definition::TypeParameter,
                        scope_id: self.current_scope_id,
                        syntax: Some((type_parameter.position(), type_parameter.id)),
                    });

                type_parameter_declaration_ids.push(type_parameter_declaration_id);
                self.resolver
                    .add_declaration_binding(type_parameter.id, type_parameter_declaration_id);
            }
        }

        let mut variant_declaration_ids =
            SmallVec::<[DeclarationId; 4]>::with_capacity(variants.child_count());

        for (index, variant) in variants.children().enumerate() {
            let EnumVariant {
                name: variant_name,
                fields: variant_fields,
            } = variant.as_component()?;

            let variant_name_str = self.source.get_file_content(&variant_name.position())?;
            let variant_symbol = self.resolver.symbols.add_symbol(variant_name_str);

            let fields = if let Some(variant_fields) = variant_fields {
                let mut field_ids = SmallVec::<[DeclarationId; 4]>::new();

                match variant_fields.node.kind {
                    SyntaxKind::StructItemTupleFields => {
                        let StructItemTupleFields { types } = variant_fields.as_component()?;

                        for field_type in types {
                            let symbol_id = self
                                .resolver
                                .symbols
                                .add_index_symbol(field_ids.len() as u32);
                            let type_id = self.visit_type(field_type)?;
                            let field_declaration_id =
                                self.resolver.declarations.add_declaration(Declaration {
                                    symbol_id,
                                    definition: Definition::Field {
                                        public: false,
                                        parent_struct: enum_declaration_id,
                                        type_id,
                                    },
                                    scope_id: self.current_scope_id,
                                    syntax: Some((field_type.position(), field_type.id)),
                                });

                            field_ids.push(field_declaration_id);
                        }
                    }
                    SyntaxKind::StructItemStructFields => {
                        let StructItemStructFields { name_type_pairs } =
                            variant_fields.as_component()?;

                        let file = self.source.get_file(variant_name.file_id())?;

                        for [field_name, field_type] in name_type_pairs {
                            let field_name_str = file.content_str(field_name.node.span)?;
                            let field_symbol_id = self.resolver.symbols.add_symbol(field_name_str);
                            let field_type_id = self.visit_type(field_type)?;
                            let field_declaration_id =
                                self.resolver.declarations.add_declaration(Declaration {
                                    symbol_id: field_symbol_id,
                                    definition: Definition::Field {
                                        public: false,
                                        parent_struct: enum_declaration_id,
                                        type_id: field_type_id,
                                    },
                                    scope_id: self.current_scope_id,
                                    syntax: Some((field_name.position(), field_name.id)),
                                });

                            field_ids.push(field_declaration_id);
                        }
                    }
                    _ => {}
                }

                self.resolver
                    .declarations
                    .add_declaration_members(field_ids)
            } else {
                DeclarationMembers::default()
            };

            let variant_declaration_id = self.resolver.declarations.add_declaration(Declaration {
                symbol_id: variant_symbol,
                definition: Definition::Variant {
                    discriminant: index as u32,
                    parent_enum: enum_declaration_id,
                    type_parameters: DeclarationMembers::default(),
                    fields,
                },
                scope_id: self.current_scope_id,
                syntax: Some((variant.position(), variant.id)),
            });

            self.resolver
                .add_declaration_binding(variant_name.id, variant_declaration_id);
            variant_declaration_ids.push(variant_declaration_id);
        }

        let type_parameters = self
            .resolver
            .declarations
            .add_declaration_members(type_parameter_declaration_ids);
        let variants = self
            .resolver
            .declarations
            .add_declaration_members(variant_declaration_ids);

        self.resolver.declarations.set_declaration(
            enum_declaration_id,
            Declaration {
                symbol_id: enum_symbol,
                definition: Definition::EnumType {
                    public,
                    type_parameters,
                    variants,
                },
                scope_id: self.current_scope_id,
                syntax: Some((reader.position(), reader.id)),
            },
        );

        self.resolver
            .add_declaration_binding(name.id, enum_declaration_id);

        Ok(())
    }

    fn visit_const_item(&mut self, reader: SyntaxReader) -> Result<(), CompileError> {
        let ConstItem {
            public,
            name,
            type_annotation,
            value,
        } = reader.as_component()?;

        self.visit_expression(value, None)?;

        let const_name_str = self.source.get_file_content(&name.position())?;
        let const_symbol_id = self.resolver.symbols.add_symbol(const_name_str);
        let type_id = self.visit_type(type_annotation)?;
        let declaration_id = self.resolver.declarations.add_declaration(Declaration {
            symbol_id: const_symbol_id,
            definition: Definition::Constant { public, type_id },
            scope_id: self.current_scope_id,
            syntax: Some((reader.position(), reader.id)),
        });

        self.resolver
            .add_declaration_binding(name.id, declaration_id);

        Ok(())
    }

    fn visit_type_item(&mut self, reader: SyntaxReader) -> Result<(), CompileError> {
        let TypeItem {
            public,
            name,
            type_parameters,
            aliased_type,
        } = reader.as_component()?;

        let mut type_parameter_declaration_ids = SmallVec::<[DeclarationId; 4]>::new();

        if let Some(type_parameters) = type_parameters {
            type_parameter_declaration_ids.reserve(type_parameters.child_count());

            for type_parameter in type_parameters.children() {
                let type_parameter_name_str =
                    self.source.get_file_content(&type_parameter.position())?;
                let type_parameter_symbol_id =
                    self.resolver.symbols.add_symbol(type_parameter_name_str);
                let type_parameter_declaration_id =
                    self.resolver.declarations.add_declaration(Declaration {
                        symbol_id: type_parameter_symbol_id,
                        definition: Definition::TypeParameter,
                        scope_id: self.current_scope_id,
                        syntax: Some((type_parameter.position(), type_parameter.id)),
                    });

                type_parameter_declaration_ids.push(type_parameter_declaration_id);
                self.resolver
                    .add_declaration_binding(type_parameter.id, type_parameter_declaration_id);
            }
        }

        let type_parameters = self
            .resolver
            .declarations
            .add_declaration_members(type_parameter_declaration_ids);
        let aliased_type_id = self.visit_type(aliased_type)?;
        let type_alias_name_str = self.source.get_file_content(&name.position())?;
        let type_alias_symbol_id = self.resolver.symbols.add_symbol(type_alias_name_str);
        let declaration_id = self.resolver.declarations.add_declaration(Declaration {
            symbol_id: type_alias_symbol_id,
            definition: Definition::TypeAlias {
                public,
                type_parameters,
                aliased_type_id,
            },
            scope_id: self.current_scope_id,
            syntax: Some((reader.position(), reader.id)),
        });

        self.resolver
            .add_declaration_binding(name.id, declaration_id);

        Ok(())
    }

    fn visit_impl_item(&mut self, reader: SyntaxReader) -> Result<(), CompileError> {
        let ImplItem {
            self_type,
            body,
            type_parameters,
            ..
        } = reader.as_component()?;

        let starting_scope_id = self.current_scope_id;
        self.current_scope_id = self.resolver.scopes.add_scope(Scope {
            kind: ScopeKind::Impl,
            parent: self.current_scope_id,
            modules: smallvec![ScopeId::CORE],
            imports: SmallVec::new(),
        });

        let mut type_parameter_declaration_ids = SmallVec::<[DeclarationId; 4]>::new();

        if let Some(type_parameters) = type_parameters {
            type_parameter_declaration_ids.reserve(type_parameters.child_count());

            for type_parameter in type_parameters.children() {
                let type_parameter_name_str =
                    self.source.get_file_content(&type_parameter.position())?;
                let type_parameter_symbol_id =
                    self.resolver.symbols.add_symbol(type_parameter_name_str);
                let type_parameter_declaration_id =
                    self.resolver.declarations.add_declaration(Declaration {
                        symbol_id: type_parameter_symbol_id,
                        definition: Definition::TypeParameter,
                        scope_id: self.current_scope_id,
                        syntax: Some((type_parameter.position(), type_parameter.id)),
                    });

                type_parameter_declaration_ids.push(type_parameter_declaration_id);
                self.resolver
                    .add_declaration_binding(type_parameter.id, type_parameter_declaration_id);
            }
        }

        let type_parameters = self
            .resolver
            .declarations
            .add_declaration_members(type_parameter_declaration_ids);

        let self_type_id = self.visit_type(self_type)?;
        let previous_self_type_id = self.current_self_type_id.replace(self_type_id);

        let mut member_declaration_ids = SmallVec::<[DeclarationId; 4]>::new();

        for child in body.children() {
            match child.node.kind {
                SyntaxKind::FunctionItem => {
                    self.visit_function_item(child)?;

                    let function_declaration_id = *self
                        .resolver
                        .get_declaration_binding(&child.children().expect_next()?.id)?;

                    member_declaration_ids.push(function_declaration_id);
                }
                SyntaxKind::ConstItem => {
                    self.visit_const_item(child)?;

                    let const_declaration_id = *self
                        .resolver
                        .get_declaration_binding(&child.children().expect_next()?.id)?;

                    member_declaration_ids.push(const_declaration_id);
                }
                _ => {
                    return Err(CompileError::ExpectedSyntaxKinds {
                        expected: &[SyntaxKind::FunctionItem, SyntaxKind::ConstItem],
                        found: child.node.kind,
                    });
                }
            }
        }

        self.current_self_type_id = previous_self_type_id;

        let declarations = self
            .resolver
            .declarations
            .add_declaration_members(member_declaration_ids);

        let impl_symbol_id = self.resolver.symbols.add_impl_symbol();
        let impl_declaration_id = self.resolver.declarations.add_declaration(Declaration {
            symbol_id: impl_symbol_id,
            definition: Definition::InherentImplementation {
                type_parameters,
                declarations,
            },
            scope_id: starting_scope_id,
            syntax: Some((reader.position(), reader.id)),
        });

        self.resolver
            .add_declaration_binding(reader.id, impl_declaration_id);
        self.resolver
            .add_scope_binding(body.id, self.current_scope_id);

        self.current_scope_id = starting_scope_id;

        Ok(())
    }

    fn visit_impl_trait_item(&mut self, reader: SyntaxReader) -> Result<(), CompileError> {
        let ImplTraitItem {
            self_type,
            body,
            trait_path,
            type_parameters,
            type_arguments,
            ..
        } = reader.as_component()?;

        let starting_scope_id = self.current_scope_id;
        self.current_scope_id = self.resolver.scopes.add_scope(Scope {
            kind: ScopeKind::Impl,
            parent: self.current_scope_id,
            modules: smallvec![ScopeId::CORE],
            imports: SmallVec::new(),
        });

        let mut type_parameter_declaration_ids = SmallVec::<[DeclarationId; 4]>::new();

        if let Some(type_parameters) = type_parameters {
            type_parameter_declaration_ids.reserve(type_parameters.child_count());

            for type_parameter in type_parameters.children() {
                let type_parameter_name_str =
                    self.source.get_file_content(&type_parameter.position())?;
                let type_parameter_symbol_id =
                    self.resolver.symbols.add_symbol(type_parameter_name_str);
                let type_parameter_declaration_id =
                    self.resolver.declarations.add_declaration(Declaration {
                        symbol_id: type_parameter_symbol_id,
                        definition: Definition::TypeParameter,
                        scope_id: self.current_scope_id,
                        syntax: Some((type_parameter.position(), type_parameter.id)),
                    });

                type_parameter_declaration_ids.push(type_parameter_declaration_id);
                self.resolver
                    .add_declaration_binding(type_parameter.id, type_parameter_declaration_id);
            }
        }

        let type_parameters = self
            .resolver
            .declarations
            .add_declaration_members(type_parameter_declaration_ids);

        let trait_declaration_id = self.visit_path(trait_path, Visibility::Module)?;

        let trait_type_arguments = if let Some(type_arguments) = type_arguments {
            let mut type_ids = SmallVec::<[TypeId; 4]>::with_capacity(type_arguments.child_count());

            for type_argument in type_arguments.children() {
                type_ids.push(self.visit_type(type_argument)?);
            }

            self.resolver.types.add_type_members(type_ids)
        } else {
            TypeMembers::default()
        };

        let impl_declaration_id = self.resolver.declarations.reserve_declaration_id();

        let self_type_id = self.visit_type(self_type)?;
        let previous_self_type_id = self.current_self_type_id.replace(self_type_id);

        let mut member_declaration_ids = SmallVec::<[DeclarationId; 8]>::new();

        for child in body.children() {
            match child.node.kind {
                SyntaxKind::FunctionItem => {
                    self.visit_function_item(child)?;

                    let function_declaration_id = *self
                        .resolver
                        .get_declaration_binding(&child.children().expect_next()?.id)?;

                    member_declaration_ids.push(function_declaration_id);
                }
                SyntaxKind::ConstItem => {
                    self.visit_const_item(child)?;

                    let const_declaration_id = *self
                        .resolver
                        .get_declaration_binding(&child.children().expect_next()?.id)?;

                    member_declaration_ids.push(const_declaration_id);
                }
                SyntaxKind::TypeItem => {
                    let TypeItem {
                        public: type_public,
                        name: type_name,
                        type_parameters,
                        aliased_type,
                    } = child.as_component()?;

                    let mut type_parameter_declaration_ids = SmallVec::<[DeclarationId; 4]>::new();

                    if let Some(type_parameters) = type_parameters {
                        type_parameter_declaration_ids.reserve(type_parameters.child_count());

                        for type_parameter in type_parameters.children() {
                            let type_parameter_name_str =
                                self.source.get_file_content(&type_parameter.position())?;
                            let type_parameter_symbol_id =
                                self.resolver.symbols.add_symbol(type_parameter_name_str);
                            let type_parameter_declaration_id =
                                self.resolver.declarations.add_declaration(Declaration {
                                    symbol_id: type_parameter_symbol_id,
                                    definition: Definition::TypeParameter,
                                    scope_id: self.current_scope_id,
                                    syntax: Some((type_parameter.position(), type_parameter.id)),
                                });

                            type_parameter_declaration_ids.push(type_parameter_declaration_id);
                            self.resolver.add_declaration_binding(
                                type_parameter.id,
                                type_parameter_declaration_id,
                            );
                        }
                    }

                    let type_parameters = self
                        .resolver
                        .declarations
                        .add_declaration_members(type_parameter_declaration_ids);
                    let aliased_type_id = self.visit_type(aliased_type)?;
                    let type_name_str = self.source.get_file_content(&type_name.position())?;
                    let type_symbol_id = self.resolver.symbols.add_symbol(type_name_str);
                    let type_declaration_id =
                        self.resolver.declarations.add_declaration(Declaration {
                            symbol_id: type_symbol_id,
                            definition: Definition::AssociatedType {
                                public: type_public,
                                parent: impl_declaration_id,
                                type_parameters,
                                aliased_type_id,
                            },
                            scope_id: self.current_scope_id,
                            syntax: Some((child.position(), child.id)),
                        });

                    self.resolver
                        .add_declaration_binding(type_name.id, type_declaration_id);
                    member_declaration_ids.push(type_declaration_id);
                }
                _ => {
                    return Err(CompileError::ExpectedSyntaxKinds {
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

        self.current_self_type_id = previous_self_type_id;

        let declarations = self
            .resolver
            .declarations
            .add_declaration_members(member_declaration_ids);

        let impl_symbol_id = self.resolver.symbols.add_impl_symbol();

        self.resolver.declarations.set_declaration(
            impl_declaration_id,
            Declaration {
                symbol_id: impl_symbol_id,
                definition: Definition::TraitImplementation {
                    type_parameters,
                    trait_declaration_id: Some(trait_declaration_id),
                    trait_type_arguments,
                    declarations,
                },
                scope_id: starting_scope_id,
                syntax: Some((reader.position(), reader.id)),
            },
        );

        self.resolver
            .add_declaration_binding(reader.id, impl_declaration_id);
        self.resolver
            .add_scope_binding(body.id, self.current_scope_id);

        self.current_scope_id = starting_scope_id;

        Ok(())
    }

    fn visit_trait_item(&mut self, reader: SyntaxReader) -> Result<(), CompileError> {
        let TraitItem {
            public,
            name,
            body,
            type_parameters,
            supertraits,
            ..
        } = reader.as_component()?;

        let starting_scope_id = self.current_scope_id;
        self.current_scope_id = self.resolver.scopes.add_scope(Scope {
            kind: ScopeKind::Trait,
            parent: self.current_scope_id,
            modules: smallvec![ScopeId::CORE],
            imports: SmallVec::new(),
        });

        let mut type_parameter_declaration_ids = SmallVec::<[DeclarationId; 4]>::new();

        if let Some(type_parameters) = type_parameters {
            type_parameter_declaration_ids.reserve(type_parameters.child_count());

            for type_parameter in type_parameters.children() {
                let type_parameter_name_str =
                    self.source.get_file_content(&type_parameter.position())?;
                let type_parameter_symbol_id =
                    self.resolver.symbols.add_symbol(type_parameter_name_str);
                let type_parameter_declaration_id =
                    self.resolver.declarations.add_declaration(Declaration {
                        symbol_id: type_parameter_symbol_id,
                        definition: Definition::TypeParameter,
                        scope_id: self.current_scope_id,
                        syntax: Some((type_parameter.position(), type_parameter.id)),
                    });

                type_parameter_declaration_ids.push(type_parameter_declaration_id);
                self.resolver
                    .add_declaration_binding(type_parameter.id, type_parameter_declaration_id);
            }
        }

        let type_parameters = self
            .resolver
            .declarations
            .add_declaration_members(type_parameter_declaration_ids);

        let mut supertrait_declaration_ids = SmallVec::<[DeclarationId; 4]>::new();

        if let Some(supertraits) = supertraits {
            let TraitBounds { bounds } = supertraits.as_component()?;

            for bound in bounds {
                let supertrait_id = self.visit_path(bound, Visibility::Module)?;
                supertrait_declaration_ids.push(supertrait_id);
            }
        }

        let supertraits = self
            .resolver
            .declarations
            .add_declaration_members(supertrait_declaration_ids);

        let trait_declaration_id = self.resolver.declarations.reserve_declaration_id();

        let self_type_declaration_id = self.resolver.declarations.add_declaration(Declaration {
            symbol_id: self.resolver.symbols.add_self_symbol(),
            definition: Definition::TypeParameter,
            scope_id: self.current_scope_id,
            syntax: None,
        });
        let self_type_id = self.resolver.types.add_type(Type::Generic {
            declaration_id: self_type_declaration_id,
        });
        let previous_self_type_id = self.current_self_type_id.replace(self_type_id);

        let mut member_declaration_ids = SmallVec::<[DeclarationId; 8]>::new();

        for child in body.children() {
            match child.node.kind {
                SyntaxKind::TraitMethod => {
                    let TraitMethod {
                        public: method_public,
                        name: method_name,
                        parameters: method_parameters,
                        body: method_body,
                        return_type: method_return_type,
                        ..
                    } = child.as_component()?;
                    let FunctionParameters {
                        value_parameters: method_value_parameters,
                        type_parameters: method_type_parameters,
                    } = method_parameters.as_component()?;

                    let method_scope_id = self.resolver.scopes.add_scope(Scope {
                        kind: ScopeKind::Function,
                        parent: self.current_scope_id,
                        modules: smallvec![ScopeId::CORE],
                        imports: SmallVec::new(),
                    });
                    let previous_scope_id = self.current_scope_id;
                    self.current_scope_id = method_scope_id;

                    let mut type_parameter_declaration_ids = SmallVec::<[DeclarationId; 4]>::new();

                    if let Some(method_type_parameters) = method_type_parameters {
                        type_parameter_declaration_ids
                            .reserve(method_type_parameters.child_count());

                        for type_parameter in method_type_parameters.children() {
                            let type_parameter_name_str =
                                self.source.get_file_content(&type_parameter.position())?;
                            let type_parameter_symbol_id =
                                self.resolver.symbols.add_symbol(type_parameter_name_str);
                            let type_parameter_declaration_id =
                                self.resolver.declarations.add_declaration(Declaration {
                                    symbol_id: type_parameter_symbol_id,
                                    definition: Definition::TypeParameter,
                                    scope_id: method_scope_id,
                                    syntax: Some((type_parameter.position(), type_parameter.id)),
                                });

                            type_parameter_declaration_ids.push(type_parameter_declaration_id);
                            self.resolver.add_declaration_binding(
                                type_parameter.id,
                                type_parameter_declaration_id,
                            );
                        }
                    }

                    let value_parameters = {
                        let mut type_ids = SmallVec::<[TypeId; 4]>::with_capacity(
                            method_value_parameters.child_count(),
                        );

                        for [parameter_name, parameter_type] in
                            method_value_parameters.children().array_chunks()
                        {
                            let parameter_name_str =
                                self.source.get_file_content(&parameter_name.position())?;
                            let parameter_symbol_id =
                                self.resolver.symbols.add_symbol(parameter_name_str);
                            let parameter_type_id = self.visit_type(parameter_type)?;
                            let parameter_declaration_id =
                                self.resolver.declarations.add_declaration(Declaration {
                                    symbol_id: parameter_symbol_id,
                                    definition: Definition::Local {
                                        mutable: false,
                                        shadowed: None,
                                        type_id: parameter_type_id,
                                    },
                                    scope_id: method_scope_id,
                                    syntax: Some((parameter_name.position(), parameter_name.id)),
                                });

                            self.resolver.add_declaration_binding(
                                parameter_name.id,
                                parameter_declaration_id,
                            );

                            if let Ok(Type::Slice { declaration_id, .. }) =
                                self.resolver.types.get_type(parameter_type_id)
                            {
                                type_parameter_declaration_ids.push(*declaration_id);
                            }

                            type_ids.push(parameter_type_id);
                        }

                        self.resolver.types.add_type_members(type_ids)
                    };

                    let type_parameters = self
                        .resolver
                        .declarations
                        .add_declaration_members(type_parameter_declaration_ids);
                    let return_type_id = if let Some(return_type) = method_return_type {
                        self.visit_type(return_type)?
                    } else {
                        TypeId::UNIT
                    };
                    let method_name_str = self.source.get_file_content(&method_name.position())?;
                    let method_symbol_id = self.resolver.symbols.add_symbol(method_name_str);
                    let method_declaration_id =
                        self.resolver.declarations.add_declaration(Declaration {
                            symbol_id: method_symbol_id,
                            definition: Definition::Function {
                                public: method_public,
                                type_parameters,
                                value_parameters,
                                return_type_id,
                            },
                            scope_id: previous_scope_id,
                            syntax: Some((child.position(), child.id)),
                        });

                    self.resolver
                        .add_declaration_binding(method_name.id, method_declaration_id);
                    member_declaration_ids.push(method_declaration_id);

                    if let Some(method_body) = method_body {
                        self.resolver
                            .add_scope_binding(method_body.id, method_scope_id);

                        for body_child in method_body.children() {
                            if body_child.node.kind.is_statement() {
                                match self.visit_statement(body_child) {
                                    Ok(_) => {}
                                    Err(error) => self.errors.push(ErrorKind::Compile(error)),
                                }
                            } else {
                                self.visit_expression(body_child, None)?;
                            }
                        }
                    }

                    self.current_scope_id = previous_scope_id;
                }
                SyntaxKind::TraitConst => {
                    let TraitConst {
                        name: const_name,
                        type_annotation,
                        value,
                    } = child.as_component()?;

                    if let Some(value) = value {
                        self.visit_expression(value, None)?;
                    }

                    let const_name_str = self.source.get_file_content(&const_name.position())?;
                    let const_symbol_id = self.resolver.symbols.add_symbol(const_name_str);
                    let type_id = self.visit_type(type_annotation)?;
                    let const_declaration_id =
                        self.resolver.declarations.add_declaration(Declaration {
                            symbol_id: const_symbol_id,
                            definition: Definition::AssociatedConstant {
                                public: false,
                                parent: trait_declaration_id,
                                type_id,
                            },
                            scope_id: self.current_scope_id,
                            syntax: Some((child.position(), child.id)),
                        });

                    self.resolver
                        .add_declaration_binding(const_name.id, const_declaration_id);
                    member_declaration_ids.push(const_declaration_id);
                }
                SyntaxKind::TraitType => {
                    let TraitType {
                        name: type_name,
                        aliased_type,
                    } = child.as_component()?;

                    let aliased_type_id = if let Some(aliased_type) = aliased_type {
                        self.visit_type(aliased_type)?
                    } else {
                        TypeId::UNIT
                    };
                    let type_name_str = self.source.get_file_content(&type_name.position())?;
                    let type_symbol_id = self.resolver.symbols.add_symbol(type_name_str);
                    let type_declaration_id =
                        self.resolver.declarations.add_declaration(Declaration {
                            symbol_id: type_symbol_id,
                            definition: Definition::AssociatedType {
                                public: false,
                                parent: trait_declaration_id,
                                type_parameters: DeclarationMembers::default(),
                                aliased_type_id,
                            },
                            scope_id: self.current_scope_id,
                            syntax: Some((child.position(), child.id)),
                        });

                    self.resolver
                        .add_declaration_binding(type_name.id, type_declaration_id);
                    member_declaration_ids.push(type_declaration_id);
                }
                _ => {
                    return Err(CompileError::ExpectedSyntaxKinds {
                        expected: &[
                            SyntaxKind::TraitMethod,
                            SyntaxKind::TraitConst,
                            SyntaxKind::TraitType,
                        ],
                        found: child.node.kind,
                    });
                }
            }
        }

        let declarations = self
            .resolver
            .declarations
            .add_declaration_members(member_declaration_ids);

        let trait_name_str = self.source.get_file_content(&name.position())?;
        let trait_symbol_id = self.resolver.symbols.add_symbol(trait_name_str);

        self.resolver.declarations.set_declaration(
            trait_declaration_id,
            Declaration {
                symbol_id: trait_symbol_id,
                definition: Definition::Trait {
                    public,
                    inner_scope_id: self.current_scope_id,
                    type_parameters,
                    supertraits,
                    declarations,
                },
                scope_id: starting_scope_id,
                syntax: Some((reader.position(), reader.id)),
            },
        );

        self.resolver
            .add_declaration_binding(name.id, trait_declaration_id);
        self.resolver
            .add_scope_binding(body.id, self.current_scope_id);

        self.current_self_type_id = previous_self_type_id;
        self.current_scope_id = starting_scope_id;

        Ok(())
    }

    fn visit_let_statement(&mut self, reader: SyntaxReader) -> Result<(), CompileError> {
        let LetStatement {
            mutable,
            name,
            expression,
            type_notation,
        } = reader.as_component()?;

        self.visit_expression(expression, None)?;

        let identifier = self.source.get_file_content(&name.position())?;
        let symbol_id = self.resolver.symbols.add_symbol(identifier);
        let type_id = if let Some(type_notation) = type_notation {
            self.visit_type(type_notation)?
        } else {
            self.resolver.types.create_inferred_type(None)
        };
        let shadowed = self
            .resolver
            .find_declaration_in_scope(symbol_id, self.current_scope_id, Visibility::Block, &name)
            .ok()
            .map(|(declaration_id, _)| declaration_id);
        let declaration_id = self.resolver.declarations.add_declaration(Declaration {
            symbol_id,
            definition: Definition::Local {
                mutable,
                shadowed,
                type_id,
            },
            scope_id: self.current_scope_id,
            syntax: Some((name.position(), name.id)),
        });

        self.resolver
            .add_declaration_binding(name.id, declaration_id);

        Ok(())
    }

    fn visit_expression_statement(&mut self, reader: SyntaxReader) -> Result<(), CompileError> {
        let ExpressionStatement { expression } = reader.as_component()?;

        self.visit_expression(expression, None)?;

        Ok(())
    }

    fn visit_assignment_expression(&mut self, reader: SyntaxReader) -> Result<(), CompileError> {
        let AssignmentExpression { target, value } = reader.as_component()?;

        self.visit_expression(target, None)?;
        self.visit_expression(value, None)?;

        Ok(())
    }

    fn visit_compound_assignment_expression(
        &mut self,
        reader: SyntaxReader,
    ) -> Result<(), CompileError> {
        let CompoundAssignmentExpression { target, value } = reader.as_component()?;

        self.visit_expression(target, None)?;
        self.visit_expression(value, None)?;

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

    fn visit_array_expression(
        &mut self,
        reader: SyntaxReader,
        _: Option<Self::ExpressionInput>,
    ) -> Result<Self::ExpressionOutput, CompileError> {
        let ArrayExpression { elements } = reader.as_component()?;

        for element in elements {
            self.visit_expression(element, None)?;
        }

        Ok(())
    }

    fn visit_array_repeat_expression(
        &mut self,
        reader: SyntaxReader,
        _: Option<Self::ExpressionInput>,
    ) -> Result<Self::ExpressionOutput, CompileError> {
        let ArrayRepeatExpression { element, .. } = reader.as_component()?;

        self.visit_expression(element, None)?;

        Ok(())
    }

    fn visit_index_expression(
        &mut self,
        reader: SyntaxReader,
        _: Option<Self::ExpressionInput>,
    ) -> Result<Self::ExpressionOutput, CompileError> {
        let IndexExpression { list, index } = reader.as_component()?;

        self.visit_expression(list, None)?;
        self.visit_expression(index, None)?;

        Ok(())
    }

    fn visit_range_expression(
        &mut self,
        reader: SyntaxReader,
        _: Option<Self::ExpressionInput>,
    ) -> Result<Self::ExpressionOutput, CompileError> {
        let RangeExpression { start, end } = reader.as_component()?;

        self.visit_expression(start, None)?;
        self.visit_expression(end, None)?;

        Ok(())
    }

    fn visit_path_expression(
        &mut self,
        reader: SyntaxReader,
        _: Option<Self::ExpressionInput>,
    ) -> Result<Self::ExpressionOutput, CompileError> {
        debug!("Visiting path expression");
        debug_assert_eq!(reader.node.kind, SyntaxKind::PathExpression);

        let declaration_id = search_path_segments(self, reader, Visibility::Block)?;

        self.resolver
            .add_declaration_binding(reader.id, declaration_id);

        Ok(())
    }

    fn visit_struct_expression(
        &mut self,
        reader: SyntaxReader,
        _: Option<Self::ExpressionInput>,
    ) -> Result<Self::ExpressionOutput, CompileError> {
        let StructExpression { path, fields } = reader.as_component()?;
        let StructExpressionStructFields {
            name_expression_pairs,
        } = fields.as_component()?;

        let struct_declaration_id = self.visit_path(path, Visibility::Module)?;

        let struct_declaration = self
            .resolver
            .declarations
            .get_declaration(struct_declaration_id)?;
        let Definition::StructType {
            fields: struct_fields,
            ..
        } = struct_declaration.definition
        else {
            return Err(CompileError::ExpectedSyntaxKind {
                expected: SyntaxKind::StructExpression,
                found: path.node.kind,
            });
        };
        let field_declaration_ids = self
            .resolver
            .declarations
            .get_declaration_members(&struct_fields)?
            .to_vec();

        for [field_name, field_value] in name_expression_pairs {
            let field_name_str = self.source.get_file_content(&field_name.position())?;
            let field_symbol_id = self.resolver.symbols.add_symbol(field_name_str);

            for &field_id in &field_declaration_ids {
                let field_declaration = self.resolver.declarations.get_declaration(field_id)?;

                if field_declaration.symbol_id == field_symbol_id {
                    self.resolver
                        .add_declaration_binding(field_name.id, field_id);
                    break;
                }
            }

            self.visit_expression(field_value, None)?;
        }

        Ok(())
    }

    fn visit_grouped_expression(
        &mut self,
        reader: SyntaxReader,
        input: Option<Self::ExpressionInput>,
    ) -> Result<Self::ExpressionOutput, CompileError> {
        let GroupedExpression { expression } = reader.as_component()?;

        if let Some(expression) = expression {
            self.visit_expression(expression, input)
        } else {
            Ok(())
        }
    }

    fn visit_block_expression(
        &mut self,
        reader: SyntaxReader,
        _: Option<Self::ExpressionInput>,
    ) -> Result<Self::ExpressionOutput, CompileError> {
        debug!("Visiting block expression");
        debug_assert_eq!(reader.node.kind, SyntaxKind::BlockExpression);

        let block_scope_id = self.resolver.scopes.add_scope(Scope {
            kind: ScopeKind::Block,
            parent: self.current_scope_id,
            modules: smallvec![ScopeId::CORE],
            imports: SmallVec::new(),
        });
        let parent_scope_id = self.current_scope_id;
        self.current_scope_id = block_scope_id;

        for child in reader.children() {
            if child.node.kind.is_statement() {
                match self.visit_statement(child) {
                    Ok(_) => {}
                    Err(error) => self.errors.push(ErrorKind::Compile(error)),
                }
            } else {
                self.visit_expression(child, None)?;
            }
        }

        self.current_scope_id = parent_scope_id;

        self.resolver.add_scope_binding(reader.id, block_scope_id);

        Ok(())
    }

    fn visit_if_expression(
        &mut self,
        reader: SyntaxReader,
        _: Option<Self::ExpressionInput>,
    ) -> Result<Self::ExpressionOutput, CompileError> {
        let IfExpression {
            condition,
            then_branch,
            else_branch,
        } = reader.as_component()?;

        self.visit_expression(condition, None)?;
        self.visit_block_expression(then_branch, None)?;

        if let Some(else_branch) = else_branch {
            match else_branch.node.kind {
                SyntaxKind::BlockExpression => self.visit_block_expression(else_branch, None)?,
                SyntaxKind::IfExpression => self.visit_if_expression(else_branch, None)?,
                _ => {
                    return Err(CompileError::ExpectedSyntaxKinds {
                        expected: &[SyntaxKind::BlockExpression, SyntaxKind::IfExpression],
                        found: else_branch.node.kind,
                    });
                }
            }
        }

        Ok(())
    }

    fn visit_math_expression(
        &mut self,
        reader: SyntaxReader,
        _: Option<Self::ExpressionInput>,
    ) -> Result<Self::ExpressionOutput, CompileError> {
        let MathExpression { left, right } = reader.as_component()?;

        self.visit_expression(left, None)?;
        self.visit_expression(right, None)?;

        Ok(())
    }

    fn visit_comparison_expression(
        &mut self,
        reader: SyntaxReader,
        _: Option<Self::ExpressionInput>,
    ) -> Result<Self::ExpressionOutput, CompileError> {
        let ComparisonExpression { left, right } = reader.as_component()?;

        self.visit_expression(left, None)?;
        self.visit_expression(right, None)?;

        Ok(())
    }

    fn visit_logic_expression(
        &mut self,
        reader: SyntaxReader,
        _: Option<Self::ExpressionInput>,
    ) -> Result<Self::ExpressionOutput, CompileError> {
        let LogicExpression { left, right } = reader.as_component()?;

        self.visit_expression(left, None)?;
        self.visit_expression(right, None)?;

        Ok(())
    }

    fn visit_negation_expression(
        &mut self,
        reader: SyntaxReader,
        _: Option<Self::ExpressionInput>,
    ) -> Result<Self::ExpressionOutput, CompileError> {
        let NegationExpression { operand } = reader.as_component()?;

        self.visit_expression(operand, None)?;

        Ok(())
    }

    fn visit_not_expression(
        &mut self,
        reader: SyntaxReader,
        _: Option<Self::ExpressionInput>,
    ) -> Result<Self::ExpressionOutput, CompileError> {
        let NotExpression { operand } = reader.as_component()?;

        self.visit_expression(operand, None)?;

        Ok(())
    }

    fn visit_while_expression(
        &mut self,
        reader: SyntaxReader,
        _: Option<Self::ExpressionInput>,
    ) -> Result<Self::ExpressionOutput, CompileError> {
        let WhileExpression { condition, body } = reader.as_component()?;

        self.visit_expression(condition, None)?;
        self.visit_block_expression(body, None)?;

        Ok(())
    }

    fn visit_break_expression(
        &mut self,
        _: SyntaxReader,
        _: Option<Self::ExpressionInput>,
    ) -> Result<Self::ExpressionOutput, CompileError> {
        Ok(())
    }

    fn visit_call_expression(
        &mut self,
        reader: SyntaxReader,
        _: Option<Self::ExpressionInput>,
    ) -> Result<Self::ExpressionOutput, CompileError> {
        let CallExpression { callee, arguments } = reader.as_component()?;

        self.visit_expression(callee, None)?;

        for argument in arguments.children() {
            self.visit_expression(argument, None)?;
        }

        Ok(())
    }

    fn visit_field_access_expression(
        &mut self,
        reader: SyntaxReader,
        _: Option<Self::ExpressionInput>,
    ) -> Result<Self::ExpressionOutput, CompileError> {
        let FieldAccessExpression {
            operand,
            field_name,
        } = reader.as_component()?;

        self.visit_expression(operand, None)?;

        let operand_declaration_id = *self.resolver.get_declaration_binding(&operand.id)?;
        let operand_declaration = self
            .resolver
            .declarations
            .get_declaration(operand_declaration_id)?;

        let type_id = match operand_declaration.definition {
            Definition::Local { type_id, .. } | Definition::Field { type_id, .. } => type_id,
            _ => {
                return Err(CompileError::ExpectedValue {
                    node_kind: operand.node.kind,
                    position: operand.position(),
                });
            }
        };

        let operand_type = *self.resolver.types.get_type(type_id)?;

        let Type::Algebraic { declaration_id, .. } = operand_type else {
            return Err(CompileError::CannotAccessField {
                type_id,
                position: operand.position(),
            });
        };

        let struct_declaration = self.resolver.declarations.get_declaration(declaration_id)?;

        let fields = match struct_declaration.definition {
            Definition::StructType { fields, .. } | Definition::Variant { fields, .. } => fields,
            _ => {
                return Err(CompileError::CannotAccessField {
                    type_id,
                    position: operand.position(),
                });
            }
        };

        let field_name_str = self.source.get_file_content(&field_name.position())?;
        let field_symbol_id = self.resolver.symbols.add_symbol(field_name_str);

        let field_declaration_ids = self
            .resolver
            .declarations
            .get_declaration_members(&fields)?;

        let mut found_field_declaration_id = None;

        for &field_id in field_declaration_ids {
            let field_declaration = self.resolver.declarations.get_declaration(field_id)?;

            if field_declaration.symbol_id == field_symbol_id {
                found_field_declaration_id = Some(field_id);

                break;
            }
        }

        let field_declaration_id = match found_field_declaration_id {
            Some(id) => id,
            None => {
                let (member_id, _) =
                    search_impl_member(self, declaration_id, field_symbol_id, &field_name)?;

                member_id
            }
        };

        self.resolver
            .add_declaration_binding(field_name.id, field_declaration_id);

        Ok(())
    }

    fn visit_type(&mut self, reader: SyntaxReader) -> Result<Self::TypeOutput, CompileError> {
        let type_id = match reader.node.kind {
            SyntaxKind::BooleanType => TypeId::BOOLEAN,
            SyntaxKind::I8Type => TypeId::I_8,
            SyntaxKind::I16Type => TypeId::I_16,
            SyntaxKind::I32Type => TypeId::I_32,
            SyntaxKind::I64Type => TypeId::I_64,
            SyntaxKind::I128Type => TypeId::I_128,
            SyntaxKind::ISizeType => TypeId::I_SIZE,
            SyntaxKind::U8Type => TypeId::U_8,
            SyntaxKind::U16Type => TypeId::U_16,
            SyntaxKind::U32Type => TypeId::U_32,
            SyntaxKind::U64Type => TypeId::U_64,
            SyntaxKind::U128Type => TypeId::U_128,
            SyntaxKind::USizeType => TypeId::U_SIZE,
            SyntaxKind::F32Type => TypeId::F_32,
            SyntaxKind::F64Type => TypeId::F_64,
            SyntaxKind::CharacterType => TypeId::CHARACTER,
            SyntaxKind::ArrayType => {
                let ArrayType {
                    element_type,
                    length,
                } = reader.as_component()?;

                let element_type_id = self.visit_type(element_type)?;
                let length_str = self.source.get_file_content(&length.position())?;
                let length = create_usize_from_decimal(length_str)?;

                self.resolver.types.add_type(Type::Array {
                    element_type_id,
                    length,
                })
            }
            SyntaxKind::SliceType => {
                let element_type = reader.single_child()?;

                let element_type_id = self.visit_type(element_type)?;
                let slice_symbol_id = self.resolver.symbols.add_symbol("[]");
                let declaration_id = self.resolver.declarations.add_declaration(Declaration {
                    symbol_id: slice_symbol_id,
                    definition: Definition::TypeParameter,
                    scope_id: self.current_scope_id,
                    syntax: Some((reader.position(), reader.id)),
                });

                self.resolver.types.add_type(Type::Slice {
                    declaration_id,
                    element_type_id,
                })
            }
            SyntaxKind::TupleType => {
                let element_type_ids = reader
                    .children()
                    .map(|element_type| self.visit_type(element_type))
                    .try_collect::<SmallVec<[TypeId; 4]>>()?;
                let element_type_ids = self.resolver.types.add_type_members(element_type_ids);

                self.resolver
                    .types
                    .add_type(Type::Tuple { element_type_ids })
            }
            SyntaxKind::FunctionType => {
                let FunctionType {
                    value_parameter_types,
                    return_type,
                } = reader.as_component()?;

                let value_parameter_ids = value_parameter_types
                    .children()
                    .map(|parameter_type| self.visit_type(parameter_type))
                    .try_collect::<SmallVec<[TypeId; 4]>>()?;
                let value_parameters = self.resolver.types.add_type_members(value_parameter_ids);
                let return_type_id = if let Some(return_type) = return_type {
                    self.visit_type(return_type)?
                } else {
                    TypeId::UNIT
                };

                self.resolver.types.add_type(Type::Function {
                    value_parameters,
                    return_type: return_type_id,
                })
            }
            SyntaxKind::TypePath => {
                let declaration_id = search_path_segments(self, reader, Visibility::Block)?;

                self.resolver
                    .add_declaration_binding(reader.id, declaration_id);

                let declaration = self.resolver.declarations.get_declaration(declaration_id)?;

                match declaration.definition {
                    Definition::StructType {
                        public: _,
                        type_parameters: _,
                        fields: _,
                    } => self.resolver.types.add_type(Type::Algebraic {
                        declaration_id,
                        type_arguments: TypeMembers::default(),
                    }),
                    Definition::EnumType {
                        public: _,
                        type_parameters: _,
                        variants: _,
                    } => self.resolver.types.add_type(Type::Algebraic {
                        declaration_id,
                        type_arguments: TypeMembers::default(),
                    }),
                    Definition::TypeParameter => self
                        .resolver
                        .types
                        .add_type(Type::Generic { declaration_id }),
                    _ => {
                        return Err(CompileError::ExpectedTypeDeclaration(declaration_id));
                    }
                }
            }
            SyntaxKind::SelfType => self.current_self_type_id.ok_or_else(|| {
                let symbol_id = self.resolver.symbols.add_symbol("Self");

                CompileError::Undeclared {
                    symbol_id,
                    usage_position: reader.position(),
                }
            })?,
            _ => {
                return Err(CompileError::ExpectedSyntaxKinds {
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
                        SyntaxKind::SliceType,
                        SyntaxKind::TupleType,
                        SyntaxKind::FunctionType,
                        SyntaxKind::TypePath,
                        SyntaxKind::SelfType,
                    ],
                    found: reader.node.kind,
                });
            }
        };

        Ok(type_id)
    }

    fn visit_path(
        &mut self,
        path: SyntaxReader,
        visibility: Self::PathInput,
    ) -> Result<Self::PathOutput, CompileError> {
        debug!("Visiting path");
        debug_assert_eq!(path.node.kind, SyntaxKind::Path);

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
        debug_assert_eq!(simple_path.node.kind, SyntaxKind::SimplePath);

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
    let mut segments = path_expression.children();
    let first_segment = segments.expect_next()?;

    let file = binder.source.get_file(path_expression.file_id())?;

    let mut current_scope_id = binder.current_scope_id;
    let mut parent_declaration_id = None;
    let mut impl_member_scope: Option<DeclarationId> = None;

    let mut search = |segment: SyntaxReader| {
        let segment_str = file.content_str(segment.node.span)?;
        let symbol_id = binder.resolver.symbols.add_symbol(segment_str);

        let (next_declaration_id, next_definition) =
            if let Some(type_declaration_id) = impl_member_scope.take() {
                search_impl_member(binder, type_declaration_id, symbol_id, &segment)?
            } else {
                let (id, decl) = binder.resolver.find_declaration_in_scope(
                    symbol_id,
                    current_scope_id,
                    visibility,
                    &segment,
                )?;

                (id, decl.definition)
            };

        match next_definition {
            Definition::Module { inner_scope_id, .. } => {
                current_scope_id = inner_scope_id;
            }
            Definition::StructType { .. } | Definition::EnumType { .. } => {
                impl_member_scope = Some(next_declaration_id);
            }
            Definition::Field {
                parent_struct: next_parent_id,
                ..
            }
            | Definition::Variant {
                parent_enum: next_parent_id,
                ..
            } => {
                if let Some(current_parent_id) = parent_declaration_id
                    && current_parent_id != next_parent_id
                {
                    return Err(CompileError::Undeclared {
                        symbol_id,
                        usage_position: segment.position(),
                    });
                }

                parent_declaration_id = Some(next_parent_id);
            }
            _ => {
                parent_declaration_id = None;
            }
        }

        if let Some(type_arguments_node) = segment.children().next() {
            for type_argument in type_arguments_node.children() {
                binder.visit_type(type_argument)?;
            }
        }

        Ok(next_declaration_id)
    };

    let mut current_declaration_id = search(first_segment)?;

    for segment in segments {
        current_declaration_id = search(segment)?;
    }

    Ok(current_declaration_id)
}

fn search_impl_member<'a>(
    binder: &mut DeclarationBinder<'a>,
    type_declaration_id: DeclarationId,
    symbol_id: SymbolId,
    segment: &SyntaxReader,
) -> Result<(DeclarationId, Definition), CompileError> {
    let type_declaration = binder
        .resolver
        .declarations
        .get_declaration(type_declaration_id)?;
    let type_scope_id = type_declaration.scope_id;

    for (declaration_id, declaration) in binder.resolver.declarations.iter() {
        if declaration.scope_id != type_scope_id {
            continue;
        }

        match declaration.definition {
            Definition::InherentImplementation { declarations, .. }
            | Definition::TraitImplementation { declarations, .. } => {
                let member_ids = binder
                    .resolver
                    .declarations
                    .get_declaration_members(&declarations)?;

                for &member_id in member_ids {
                    let member = binder.resolver.declarations.get_declaration(member_id)?;

                    if member.symbol_id == symbol_id {
                        return Ok((member_id, member.definition));
                    }
                }

                if let Definition::TraitImplementation {
                    trait_declaration_id: Some(trait_declaration_id),
                    ..
                } = declaration.definition
                {
                    let trait_declaration = binder
                        .resolver
                        .declarations
                        .get_declaration(trait_declaration_id)?;

                    if let Definition::Trait {
                        declarations: trait_declarations,
                        ..
                    } = trait_declaration.definition
                    {
                        let trait_member_ids = binder
                            .resolver
                            .declarations
                            .get_declaration_members(&trait_declarations)?;

                        for &trait_member_id in trait_member_ids {
                            let trait_member = binder
                                .resolver
                                .declarations
                                .get_declaration(trait_member_id)?;

                            if trait_member.symbol_id == symbol_id {
                                return Ok((trait_member_id, trait_member.definition));
                            }
                        }
                    }
                }
            }
            Definition::Variant { parent_enum, .. }
                if parent_enum == type_declaration_id && declaration.symbol_id == symbol_id =>
            {
                return Ok((declaration_id, declaration.definition));
            }
            _ => continue,
        }
    }

    Err(CompileError::Undeclared {
        symbol_id,
        usage_position: segment.position(),
    })
}
