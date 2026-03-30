#[cfg(test)]
mod tests;

use smallvec::{SmallVec, smallvec};
use tracing::debug;

use crate::{
    compiler::error::CompileError,
    error::ErrorKind,
    resolver::{
        Resolver,
        declarations::{
            Declaration, DeclarationId, DeclarationMembers, Definition, ModuleKind, Visibility,
        },
        scopes::{Scope, ScopeId, ScopeKind},
        types::{Type, TypeId, TypeMembers},
    },
    source::{Position, Source, Span},
    syntax::{
        Syntax,
        components::{
            AssignmentExpression, CallExpression, ComparisonExpression,
            CompoundAssignmentExpression, EnumItem, EnumVariant, ExpressionStatement, FunctionItem,
            FunctionParameters, FunctionType, GroupedExpression, IfExpression, IndexExpression,
            LetStatement, LogicExpression, MathExpression, ModuleItem, NegationExpression,
            NotExpression, StructExpression, StructExpressionStructFields, StructItem,
            StructItemStructFields, StructItemTupleFields, SyntaxComponent, UseItem,
            WhileExpression,
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

        let type_parameters = if let Some(type_parameters) = type_parameters {
            let mut declaration_ids =
                SmallVec::<[DeclarationId; 4]>::with_capacity(type_parameters.child_count());

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

                declaration_ids.push(type_parameter_declaration_id);
                self.resolver
                    .add_declaration_binding(type_parameter.id, type_parameter_declaration_id);
            }

            self.resolver
                .declarations
                .add_declaration_members(declaration_ids)
        } else {
            DeclarationMembers::default()
        };
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

                type_ids.push(parameter_type_id);
            }

            self.resolver.types.add_type_members(type_ids)
        };
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
            .add_declaration_binding(reader.id, function_declaration_id);
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
        } = reader.as_component()?;

        let struct_name_str = self.source.get_file_content(&name.position())?;
        let struct_symbol = self.resolver.symbols.add_symbol(struct_name_str);

        let base_id = self.resolver.declarations.next_declaration_id();
        let type_parameter_count = type_parameters.map_or(0, |tp| tp.child_count()) as u32;
        let field_count = match fields.node.kind {
            SyntaxKind::StructItemTupleFields => fields.child_count() as u32,
            SyntaxKind::StructItemStructFields => (fields.child_count() / 2) as u32,
            SyntaxKind::StructItemUnit => 0,
            _ => unreachable!(),
        };

        let type_parameter_declaration_ids = if type_parameters.is_some() {
            (0..type_parameter_count)
                .map(|index| base_id.offset(index))
                .collect::<SmallVec<[DeclarationId; 4]>>()
        } else {
            SmallVec::default()
        };
        let field_declaration_ids = (0..field_count)
            .map(|index| base_id.offset(type_parameter_count + index))
            .collect::<SmallVec<[DeclarationId; 4]>>();
        let struct_declaration_id = base_id.offset(type_parameter_count + field_count);

        let type_parameters = if let Some(type_parameters) = type_parameters {
            for (index, type_parameter) in type_parameters.children().enumerate() {
                let type_parameter_name_str =
                    self.source.get_file_content(&type_parameter.position())?;
                let type_parameter_symbol_id =
                    self.resolver.symbols.add_symbol(type_parameter_name_str);
                let _type_parameter_declaration_id =
                    self.resolver.declarations.add_declaration(Declaration {
                        symbol_id: type_parameter_symbol_id,
                        definition: Definition::TypeParameter,
                        scope_id: self.current_scope_id,
                        syntax: Some((type_parameter.position(), type_parameter.id)),
                    });

                debug_assert_eq!(
                    _type_parameter_declaration_id,
                    type_parameter_declaration_ids[index]
                );

                self.resolver.add_declaration_binding(
                    type_parameter.id,
                    type_parameter_declaration_ids[index],
                );
            }

            self.resolver
                .declarations
                .add_declaration_members(type_parameter_declaration_ids)
        } else {
            DeclarationMembers::default()
        };

        match fields.node.kind {
            SyntaxKind::StructItemTupleFields => {
                let StructItemTupleFields { types } = StructItemTupleFields::from_reader(&fields)?;

                for (index, field_type) in types.enumerate() {
                    let public = field_type.node.modifier;
                    let symbol_id = self.resolver.symbols.add_index_symbol(index);
                    let type_id = self.visit_type(field_type)?;
                    let _field_declaration_id =
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

                    debug_assert_eq!(_field_declaration_id, field_declaration_ids[index]);

                    self.resolver
                        .add_declaration_binding(field_type.id, field_declaration_ids[index]);
                }
            }
            SyntaxKind::StructItemStructFields => {
                let StructItemStructFields { name_type_pairs } =
                    StructItemStructFields::from_reader(&fields)?;

                let file = self.source.get_file(name.file_id())?;

                for (index, [field_name, field_type]) in name_type_pairs.enumerate() {
                    let public = field_name.node.modifier;
                    let field_name_str = file.content_str(field_name.node.span)?;
                    let field_symbol_id = self.resolver.symbols.add_symbol(field_name_str);
                    let field_type_id = self.visit_type(field_type)?;
                    let _field_declaration_id =
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

                    debug_assert_eq!(_field_declaration_id, field_declaration_ids[index]);

                    self.resolver
                        .add_declaration_binding(field_name.id, field_declaration_ids[index]);
                }
            }
            SyntaxKind::StructItemUnit => {}
            _ => unreachable!(),
        }

        let fields = self
            .resolver
            .declarations
            .add_declaration_members(field_declaration_ids);
        let _struct_declaration_id = self.resolver.declarations.add_declaration(Declaration {
            symbol_id: struct_symbol,
            definition: Definition::StructType {
                public,
                type_parameters,
                fields,
            },
            scope_id: self.current_scope_id,
            syntax: Some((reader.position(), reader.id)),
        });

        debug_assert_eq!(_struct_declaration_id, struct_declaration_id);

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

        let base_id = self.resolver.declarations.next_declaration_id();
        let type_parameter_count = type_parameters.map_or(0, |tp| tp.child_count()) as u32;

        let mut total_child_declarations = type_parameter_count;

        for variant in variants.children() {
            total_child_declarations += 1;

            match variant.node.kind {
                SyntaxKind::EnumTupleVariant => {
                    if let Some(fields) = variant.children().nth(1) {
                        total_child_declarations += fields.child_count() as u32;
                    }
                }
                SyntaxKind::EnumStructVariant => {
                    if let Some(fields) = variant.children().nth(1) {
                        total_child_declarations += (fields.child_count() / 2) as u32;
                    }
                }
                _ => {}
            }
        }

        let enum_declaration_id = base_id.offset(total_child_declarations);

        if let Some(type_parameters) = type_parameters {
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

                self.resolver
                    .add_declaration_binding(type_parameter.id, type_parameter_declaration_id);
            }
        }

        let type_parameter_declaration_ids = (0..type_parameter_count)
            .map(|index| base_id.offset(index))
            .collect::<SmallVec<[DeclarationId; 4]>>();
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
                let field_ids = match variant_fields.node.kind {
                    SyntaxKind::StructItemTupleFields => {
                        let StructItemTupleFields { types } =
                            StructItemTupleFields::from_reader(&variant_fields)?;
                        let mut ids = SmallVec::<[DeclarationId; 4]>::new();

                        for field_type in types {
                            let symbol_id = self.resolver.symbols.add_index_symbol(ids.len());
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

                            ids.push(field_declaration_id);
                        }

                        ids
                    }
                    SyntaxKind::StructItemStructFields => {
                        let StructItemStructFields { name_type_pairs } =
                            StructItemStructFields::from_reader(&variant_fields)?;
                        let file = self.source.get_file(variant_name.file_id())?;
                        let mut ids = SmallVec::<[DeclarationId; 4]>::new();

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

                            ids.push(field_declaration_id);
                        }

                        ids
                    }
                    _ => SmallVec::default(),
                };

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
        let declared_id = self.resolver.declarations.add_declaration(Declaration {
            symbol_id: enum_symbol,
            definition: Definition::EnumType {
                public,
                type_parameters,
                variants,
            },
            scope_id: self.current_scope_id,
            syntax: Some((reader.position(), reader.id)),
        });

        debug_assert_eq!(declared_id, enum_declaration_id);
        self.resolver
            .add_declaration_binding(name.id, enum_declaration_id);

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

    fn visit_list_expression(
        &mut self,
        reader: SyntaxReader,
        _: Option<Self::ExpressionInput>,
    ) -> Result<Self::ExpressionOutput, CompileError> {
        debug!("Visiting list expression");
        debug_assert_eq!(reader.node.kind, SyntaxKind::ArrayExpression);

        for element in reader.children() {
            self.visit_expression(element, None)?;
        }

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

        self.visit_path(path, Visibility::Module)?;

        if let SyntaxKind::StructExpressionStructFields = fields.node.kind {
            let StructExpressionStructFields {
                name_expression_pairs,
            } = fields.as_component()?;

            for [_, field_value] in name_expression_pairs {
                self.visit_expression(field_value, None)?;
            }

            Ok(())
        } else {
            Err(CompileError::ExpectedSyntaxKind {
                expected: SyntaxKind::StructExpressionStructFields,
                found: fields.node.kind,
            })
        }
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

    fn visit_type(&mut self, reader: SyntaxReader) -> Result<Self::TypeOutput, CompileError> {
        debug!("Visiting type");
        debug_assert!(matches!(
            reader.node.kind,
            SyntaxKind::BooleanType
                | SyntaxKind::I8Type
                | SyntaxKind::I16Type
                | SyntaxKind::I32Type
                | SyntaxKind::I64Type
                | SyntaxKind::I128Type
                | SyntaxKind::U8Type
                | SyntaxKind::U16Type
                | SyntaxKind::U32Type
                | SyntaxKind::U64Type
                | SyntaxKind::U128Type
                | SyntaxKind::F32Type
                | SyntaxKind::F64Type
                | SyntaxKind::CharacterType
                | SyntaxKind::SliceType
                | SyntaxKind::TupleType
                | SyntaxKind::FunctionType
                | SyntaxKind::TypePath
        ),);

        let type_id = match reader.node.kind {
            SyntaxKind::BooleanType => TypeId::BOOLEAN,
            SyntaxKind::I8Type => TypeId::I_8,
            SyntaxKind::I16Type => TypeId::I_16,
            SyntaxKind::I32Type => TypeId::I_32,
            SyntaxKind::I64Type => TypeId::I_64,
            SyntaxKind::I128Type => TypeId::I_128,
            SyntaxKind::U8Type => TypeId::U_8,
            SyntaxKind::U16Type => TypeId::U_16,
            SyntaxKind::U32Type => TypeId::U_32,
            SyntaxKind::U64Type => TypeId::U_64,
            SyntaxKind::U128Type => TypeId::U_128,
            SyntaxKind::F32Type => TypeId::F_32,
            SyntaxKind::F64Type => TypeId::F_64,
            SyntaxKind::CharacterType => TypeId::CHARACTER,
            SyntaxKind::SliceType => {
                let element_type = reader.single_child()?;

                self.visit_type(element_type)?
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

    let mut search =
        |segment: SyntaxReader| {
            let segment_str = file.content_str(segment.node.span)?;
            let symbol_id = binder.resolver.symbols.add_symbol(segment_str);

            let (next_declaration_id, next_declaration) = binder
                .resolver
                .find_declaration_in_scope(symbol_id, current_scope_id, visibility, &segment)?;

            if let Definition::Module { inner_scope_id, .. } = next_declaration.definition {
                current_scope_id = inner_scope_id;
            }

            if let Definition::Field {
                parent_struct: next_parent_id,
                ..
            }
            | Definition::Variant {
                parent_enum: next_parent_id,
                ..
            } = next_declaration.definition
            {
                if let Some(current_parent_id) = parent_declaration_id
                    && current_parent_id != next_parent_id
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

            Ok(next_declaration_id)
        };

    let mut current_declaration_id = search(first_segment)?;

    for segment in segments {
        let next_declaration_id = search(segment)?;

        current_declaration_id = next_declaration_id;
    }

    Ok(current_declaration_id)
}
