#[cfg(test)]
mod tests;

use std::collections::HashSet;

use rustc_hash::FxBuildHasher;
use smallvec::{SmallVec, smallvec};
use tracing::debug;

use crate::{
    compiler::{
        error::CompileError,
        resolver::{
            Resolver,
            declarations::{
                Declaration, DeclarationId, Declarations, Definition,
                ModuleKind, Visibility,
            },
            scopes::{Scope, ScopeFrame, ScopeId, ScopeKind, Scopes},
            symbols::{SymbolId, Symbols},
            types::{Type, TypeId, TypeMembers},
        },
        value_creation::create_usize_from_decimal,
    },
    error::ErrorKind,
    source::{Position, Source, Span},
    syntax::{
        Syntax, SyntaxId,
        components::{
            ArrayExpression, ArrayRepeatExpression, ArrayType, AssignmentExpression,
            CallExpression, ComparisonExpression, ConstItem, EnumItem, EnumItemTupleVariant,
            EnumNamedFieldsVariant, EnumUnitVariant, ExpressionStatement, FieldAccessExpression,
            FnItem, FunctionType, GroupedExpression, IfExpression, ImplItem, IndexExpression,
            LetStatement, LogicExpression, MathExpression, ModItem, NamedFields,
            NegationExpression, NotExpression, RangeExpression, Root, StructExpression,
            StructExpressionStructFields, StructItem, SyntaxComponent, TraitItem, TupleFields,
            TypeItem, UseItem, ValueParameters, WhileExpression,
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

    scope_search: HashSet<ScopeId, FxBuildHasher>,

    current_self_type_id: Option<TypeId>,

    current_scope_id: ScopeId,

    scope_stack: Vec<ScopeFrame>,

    scoped_declarations: Vec<(SymbolId, DeclarationId)>,
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
            scope_search: HashSet::default(),
            current_self_type_id: None,
            current_scope_id: starting_scope_id,
            scope_stack: Vec::new(),
            scoped_declarations: Vec::new(),
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
        let scope_id = self
            .resolver
            .scopes
            .enter_scope(kind, self.current_scope_id);
        self.current_scope_id = scope_id;

        self.scope_stack.push(ScopeFrame {
            scope_id,
            type_entries_start: self.scoped_declarations.len() as u32,
        });
    }

    fn exit_scope(&mut self) {
        if let Some(frame) = self.scope_stack.pop() {
            let parent = self.resolver.scopes.get_scope(frame.scope_id).parent;

            self.resolver.scopes.exit_scope(
                frame.scope_id,
                self.scoped_declarations
                    .drain(frame.type_entries_start as usize..),
            );

            self.current_scope_id = parent;
        }
    }

    fn add_scoped_declaration(&mut self, declaration: Declaration) -> DeclarationId {
        let symbol_id = declaration.symbol_id;

        debug_assert_eq!(declaration.scope_id, self.current_scope_id);

        let declaration_id = self.resolver.declarations.add_declaration(declaration);

        self.scoped_declarations.push((symbol_id, declaration_id));

        declaration_id
    }

    fn reserve_scoped_declaration(
        &mut self,
        symbol_id: SymbolId,
        syntax: Option<(Position, SyntaxId)>,
    ) -> DeclarationId {
        let declaration_id = self.resolver.declarations.reserve_declaration_id(
            symbol_id,
            self.current_scope_id,
            syntax,
        );

        self.scoped_declarations.push((symbol_id, declaration_id));

        declaration_id
    }

    fn find_declaration_in_scope(
        &mut self,
        symbol_id: SymbolId,
        start_scope_id: ScopeId,
    ) -> Option<(DeclarationId, &Declaration)> {
        let mut current_scope_id = start_scope_id;
        let mut block_blocked = false;
        let mut module_blocked = false;
        let mut type_blocked = false;

        loop {
            if current_scope_id == ScopeId::NONE {
                self.scope_search.clear();

                return None;
            }

            if !self.scope_search.insert(current_scope_id) {
                let parent = self.resolver.scopes.get_scope(current_scope_id).parent;
                current_scope_id = parent;

                continue;
            }

            let found = lookup_symbol(
                &self.scope_stack,
                &self.scoped_declarations,
                &self.resolver.scopes,
                current_scope_id,
                symbol_id,
            );

            if let Some(declaration_id) = found {
                self.scope_search.clear();

                if let Ok(declaration) = self.resolver.declarations.get_declaration(declaration_id)
                {
                    let is_blocked = match declaration.definition.visibility() {
                        Visibility::Block => block_blocked,
                        Visibility::Module => module_blocked,
                        Visibility::Type => type_blocked,
                    };

                    if is_blocked {
                        return None;
                    }

                    return Some((declaration_id, declaration));
                }

                return None;
            }

            let scope = self.resolver.scopes.get_scope(current_scope_id);
            block_blocked |= is_barrier(scope.kind, Visibility::Block);
            module_blocked |= is_barrier(scope.kind, Visibility::Module);
            type_blocked |= is_barrier(scope.kind, Visibility::Type);
            current_scope_id = scope.parent;
        }
    }

    fn find_declaration_direct(
        &self,
        scope_id: ScopeId,
        symbol_id: SymbolId,
    ) -> Option<(DeclarationId, &Declaration)> {
        let declaration_id = lookup_symbol(
            &self.scope_stack,
            &self.scoped_declarations,
            &self.resolver.scopes,
            scope_id,
            symbol_id,
        )?;

        self.resolver
            .declarations
            .get_declaration(declaration_id)
            .ok()
            .map(|declaration| (declaration_id, declaration))
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

        let module_name_str = self.source.get_content(&name.position())?;
        let module_symbol_id = self.resolver.symbols.add_symbol(module_name_str);

        let module_declaration_id =
            self.reserve_scoped_declaration(module_symbol_id, Some((name.position(), reader.id)));

        self.enter_scope(ScopeKind::Module);

        let inner_scope_id = self.current_scope_id;

        if let Some(module_body) = body {
            self.resolver
                .add_scope_binding(module_body.id, inner_scope_id);
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
                    inner_scope_id,
                },
            );
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

            self.resolver
                .add_declaration_binding(name.id, module_declaration_id);

            let module_root = self.syntax.get_tree(module_file_id)?.root()?;

            self.bind_root(module_root)?;

            self.exit_scope();

            self.resolver.declarations.set_reserved_declaration(
                module_declaration_id,
                Definition::Module {
                    public,
                    kind: ModuleKind::File {
                        file_id: module_file_id,
                    },
                    inner_scope_id,
                },
            );
        }

        Ok(())
    }

    fn bind_use_item(&mut self, reader: SyntaxReader) -> Result<(), CompileError> {
        let UseItem { public, path } = reader.as_component()?;
        fn search_enum_variants(
            target_str: &str,
            parent_enum_id: DeclarationId,
            variant_entries: &[(SymbolId, DeclarationId)],
            symbols: &Symbols,
            declarations: &Declarations,
        ) -> Result<Option<DeclarationId>, CompileError> {
            for &(_, variant_id) in variant_entries {
                let variant_declaration = declarations.get_declaration(variant_id)?;
                let variant_symbol = symbols.get_symbol(&variant_declaration.symbol_id)?;
                let variant_parent_id = if let Definition::Variant {
                    enum_declaration_id,
                    ..
                } = variant_declaration.definition
                {
                    enum_declaration_id
                } else {
                    return Err(CompileError::ExpectedVariantDefinition(variant_id));
                };

                if variant_symbol == target_str && variant_parent_id == parent_enum_id {
                    return Ok(Some(variant_id));
                }
            }

            Ok(None)
        }

        let start = reader.node.span.start();
        let file = self.source.get_code(path.file_id())?;

        let mut direct_scope: Option<ScopeId> = None;
        let mut current_declaration_id = None;
        let mut current_end = start;
        let mut symbol_id = SymbolId::PLACEHOLDER;

        let mut path_segments = path.children();

        while let Some(segment) = path_segments.next() {
            let segment_str = file.get_str(segment.node.span)?;
            let segment_symbol_id = self.resolver.symbols.add_symbol(segment_str);
            let (declaration_id, declaration) = if let Some(scope_id) = direct_scope {
                self.find_declaration_direct(scope_id, segment_symbol_id)
            } else {
                self.find_declaration_in_scope(segment_symbol_id, self.current_scope_id)
            }
            .ok_or(CompileError::Undeclared {
                symbol_id: segment_symbol_id,
                usage_position: segment.position(),
            })?;

            current_declaration_id = Some(declaration_id);
            current_end = segment.node.span.end();
            symbol_id = segment_symbol_id;

            match declaration.definition {
                Definition::Module {
                    public: true,
                    inner_scope_id,
                    ..
                } => {
                    direct_scope = Some(inner_scope_id);
                }
                Definition::EnumType {
                    public: true,
                    variants,
                    ..
                } => {
                    let variant_entries = self.resolver.scopes.get_namespace_entries(variants);

                    if let Some(next_segment) = path_segments.next()
                        && let Some(found_variant_id) = search_enum_variants(
                            file.get_str(next_segment.node.span)?,
                            declaration_id,
                            variant_entries,
                            &self.resolver.symbols,
                            &self.resolver.declarations,
                        )?
                    {
                        current_declaration_id = Some(found_variant_id);
                        current_end = next_segment.node.span.end();
                        symbol_id = segment_symbol_id;

                        break;
                    }
                }
                _ => {
                    return Err(CompileError::CannotImport {
                        declaration_id,
                        position: segment.position(),
                    });
                }
            }
        }

        if let Some(current_declaration_id) = current_declaration_id {
            let use_declaration_id = self.add_scoped_declaration(Declaration {
                symbol_id,
                definition: Definition::Use {
                    public,
                    source_declaration_id: current_declaration_id,
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

        let function_name_str = self.source.get_content(&name.position())?;
        let function_symbol_id = self.resolver.symbols.add_symbol(function_name_str);

        let function_declaration_id = self
            .reserve_scoped_declaration(function_symbol_id, Some((reader.position(), reader.id)));

        let type_parameters_scope_id = if let Some(type_parameters) = type_parameters {
            self.bind_type_parameters(type_parameters)?
        } else {
            ScopeId::NONE
        };

        let value_parameters_scope_id = if let Some(value_parameters) = value_parameters {
            let ValueParameters { name_type_pairs } = value_parameters.as_component()?;

            self.enter_scope(ScopeKind::ValueParameters);

            for (parameter_name, parameter_type) in name_type_pairs {
                let parameter_name_str = self.source.get_content(&parameter_name.position())?;
                let parameter_symbol_id = self.resolver.symbols.add_symbol(parameter_name_str);
                let parameter_type_id = self.get_explicit_type(parameter_type)?;
                let parameter_declaration_id = self.add_scoped_declaration(Declaration {
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
            }

            self.current_scope_id
        } else {
            ScopeId::NONE
        };

        let return_type_id = if let Some(return_type) = return_type {
            self.get_explicit_type(return_type)?
        } else {
            TypeId::UNIT
        };

        self.enter_scope(ScopeKind::Function);

        if let Some(body) = body {
            self.resolver
                .add_scope_binding(body.id, self.current_scope_id);

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

        if value_parameters_scope_id != ScopeId::NONE {
            self.exit_scope();
        }

        if type_parameters_scope_id != ScopeId::NONE {
            self.exit_scope();
        }

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

        let struct_name_str = self.source.get_content(&name.position())?;
        let struct_symbol_id = self.resolver.symbols.add_symbol(struct_name_str);
        let struct_declaration_id =
            self.reserve_scoped_declaration(struct_symbol_id, Some((reader.position(), reader.id)));

        let type_parameters_scope_id = if let Some(type_parameters) = type_parameters {
            self.bind_type_parameters(type_parameters)?
        } else {
            ScopeId::NONE
        };

        self.enter_scope(ScopeKind::TypeTraitOrImpl);

        let fields = if let Some(fields) = fields {
            match fields.node.kind {
                SyntaxKind::TupleFields => {
                    let TupleFields { types } = TupleFields::from_reader(&fields)?;

                    for (index, field_type) in types.enumerate() {
                        let public = field_type.node.flags.get_flag(SyntaxFlags::PUBLIC);
                        let symbol_id = self.resolver.symbols.add_index_symbol(index as u32);
                        let type_id = self.get_explicit_type(field_type)?;
                        self.add_scoped_declaration(Declaration {
                            symbol_id,
                            definition: Definition::Field {
                                public,
                                parent_struct: struct_declaration_id,
                                type_id,
                            },
                            scope_id: self.current_scope_id,
                            syntax: Some((field_type.position(), field_type.id)),
                        });
                    }
                }
                SyntaxKind::NamedFields => {
                    let NamedFields { name_type_pairs } = NamedFields::from_reader(&fields)?;

                    let file = self.source.get_code(name.file_id())?;

                    for (field_name, field_type) in name_type_pairs {
                        let public = field_name.node.flags.get_flag(SyntaxFlags::PUBLIC);
                        let field_name_str = file.get_str(field_name.node.span)?;
                        let field_symbol_id = self.resolver.symbols.add_symbol(field_name_str);
                        let field_type_id = self.get_explicit_type(field_type)?;
                        let field_declaration_id = self.add_scoped_declaration(Declaration {
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
                    }
                }
                _ => {
                    return Err(CompileError::UnexpectedSyntax {
                        expected: &[SyntaxKind::TupleFields, SyntaxKind::NamedFields],
                        found: fields.node.kind,
                    });
                }
            }

            self.current_scope_id
        } else {
            ScopeId::NONE
        };

        self.exit_scope();

        if type_parameters_scope_id != ScopeId::NONE {
            self.exit_scope();
        }

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

        let enum_name_str = self.source.get_content(&name.position())?;
        let enum_symbol_id = self.resolver.symbols.add_symbol(enum_name_str);
        let enum_declaration_id =
            self.reserve_scoped_declaration(enum_symbol_id, Some((reader.position(), reader.id)));

        let type_parameters_scope_id = if let Some(type_parameters) = type_parameters {
            self.bind_type_parameters(type_parameters)?
        } else {
            ScopeId::NONE
        };

        self.enter_scope(ScopeKind::TypeTraitOrImpl);

        for (index, variant) in variants.children().enumerate() {
            self.bind_enum_variant(variant, enum_declaration_id, index as u16)?;
        }

        let variants_scope_id = self.current_scope_id;

        self.exit_scope();

        if type_parameters_scope_id != ScopeId::NONE {
            self.exit_scope();
        }

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
    ) -> Result<ScopeId, CompileError> {
        self.enter_scope(ScopeKind::TypeParameters);

        let type_parameters_scope_id = self.current_scope_id;

        for type_parameter in type_parameters_reader.children() {
            let type_parameter_name_str = self.source.get_content(&type_parameter.position())?;
            let type_parameter_symbol_id =
                self.resolver.symbols.add_symbol(type_parameter_name_str);
            let type_parameter_declaration_id = self.add_scoped_declaration(Declaration {
                symbol_id: type_parameter_symbol_id,
                definition: Definition::TypeParameter,
                scope_id: self.current_scope_id,
                syntax: Some((type_parameter.position(), type_parameter.id)),
            });

            self.resolver
                .add_declaration_binding(type_parameter.id, type_parameter_declaration_id);
        }

        Ok(type_parameters_scope_id)
    }

    fn bind_enum_variant(
        &mut self,
        reader: SyntaxReader,
        enum_declaration_id: DeclarationId,
        discriminant: u16,
    ) -> Result<DeclarationId, CompileError> {
        let file = self.source.get_code(reader.file_id())?;

        match reader.node.kind {
            SyntaxKind::EnumUnitVariant => {
                let EnumUnitVariant { name } = reader.as_component()?;

                let variant_name_str = file.get_str(name.node.span)?;
                let variant_symbol_id = self.resolver.symbols.add_symbol(variant_name_str);
                let variant_declaration_id = self.add_scoped_declaration(Declaration {
                    symbol_id: variant_symbol_id,
                    definition: Definition::Variant {
                        discriminant,
                        enum_declaration_id,
                        fields: ScopeId::NONE,
                    },
                    scope_id: self.current_scope_id,
                    syntax: Some((name.position(), name.id)),
                });

                self.resolver
                    .add_declaration_binding(name.id, variant_declaration_id);

                Ok(variant_declaration_id)
            }
            SyntaxKind::EnumTupleFieldsVariant => {
                let EnumItemTupleVariant { name, tuple_fields } = reader.as_component()?;
                let TupleFields { types } = tuple_fields.as_component()?;

                let variant_name_str = file.get_str(name.node.span)?;
                let variant_symbol_id = self.resolver.symbols.add_symbol(variant_name_str);
                let variant_declaration_id = self.reserve_scoped_declaration(
                    variant_symbol_id,
                    Some((reader.position(), reader.id)),
                );

                self.enter_scope(ScopeKind::TypeTraitOrImpl);

                for (index, field_type) in types.enumerate() {
                    let symbol_id = self.resolver.symbols.add_index_symbol(index as u32);
                    let type_id = self.get_explicit_type(field_type)?;
                    self.add_scoped_declaration(Declaration {
                        symbol_id,
                        definition: Definition::Field {
                            public: false,
                            parent_struct: variant_declaration_id,
                            type_id,
                        },
                        scope_id: self.current_scope_id,
                        syntax: Some((field_type.position(), field_type.id)),
                    });
                }

                let fields_scope_id = self.current_scope_id;
                self.exit_scope();

                self.resolver.declarations.set_reserved_declaration(
                    variant_declaration_id,
                    Definition::Variant {
                        discriminant,
                        enum_declaration_id,
                        fields: fields_scope_id,
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
                let variant_declaration_id = self.reserve_scoped_declaration(
                    variant_symbol_id,
                    Some((name.position(), name.id)),
                );

                self.enter_scope(ScopeKind::TypeTraitOrImpl);

                for (field_name, field_type) in name_type_pairs {
                    let field_name_str = file.get_str(field_name.node.span)?;
                    let field_symbol_id = self.resolver.symbols.add_symbol(field_name_str);
                    let field_type_id = self.get_explicit_type(field_type)?;
                    self.add_scoped_declaration(Declaration {
                        symbol_id: field_symbol_id,
                        definition: Definition::Field {
                            public: false,
                            parent_struct: variant_declaration_id,
                            type_id: field_type_id,
                        },
                        scope_id: self.current_scope_id,
                        syntax: Some((field_name.position(), field_name.id)),
                    });
                }

                let fields_scope_id = self.current_scope_id;
                self.exit_scope();

                self.resolver.declarations.set_reserved_declaration(
                    variant_declaration_id,
                    Definition::Variant {
                        discriminant,
                        enum_declaration_id,
                        fields: fields_scope_id,
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

        let const_name_str = self.source.get_content(&name.position())?;
        let const_symbol_id = self.resolver.symbols.add_symbol(const_name_str);
        let type_id = self.get_explicit_type(type_notation)?;
        let const_declaration_id = self.add_scoped_declaration(Declaration {
            symbol_id: const_symbol_id,
            definition: Definition::Constant { public, type_id },
            scope_id: self.current_scope_id,
            syntax: Some((reader.position(), reader.id)),
        });

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
                file_id: reader.file_id(),
                syntax_id: reader.id,
            });
        };

        let type_parameters_scope_id = if let Some(type_parameters) = type_parameters {
            self.bind_type_parameters(type_parameters)?
        } else {
            ScopeId::NONE
        };

        let aliased_type_id = self.get_explicit_type(aliased_type)?;

        if type_parameters_scope_id != ScopeId::NONE {
            self.exit_scope();
        }

        let type_alias_name_str = self.source.get_content(&name.position())?;
        let type_alias_symbol_id = self.resolver.symbols.add_symbol(type_alias_name_str);
        let declaration_id = self.add_scoped_declaration(Declaration {
            symbol_id: type_alias_symbol_id,
            definition: Definition::TypeAlias {
                public,
                type_parameters: type_parameters_scope_id,
                aliased_type_id,
            },
            scope_id: self.current_scope_id,
            syntax: Some((reader.position(), reader.id)),
        });

        self.resolver
            .add_declaration_binding(name.id, declaration_id);

        Ok(declaration_id)
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

        let type_parameters_scope_id = if let Some(type_parameters) = type_parameters {
            self.bind_type_parameters(type_parameters)?
        } else {
            ScopeId::NONE
        };

        self.enter_scope(ScopeKind::TypeTraitOrImpl);

        let trait_declaration_id = if let Some(trait_path) = trait_path {
            Some(self.bind_path(trait_path)?)
        } else {
            None
        };
        let self_type_id = self.get_explicit_type(self_name)?;
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
                    self.bind_type_item(child)?;
                }
                _ => {
                    return Err(CompileError::UnexpectedSyntax {
                        expected: &[SyntaxKind::FnItem, SyntaxKind::ConstItem],
                        found: child.node.kind,
                    });
                }
            }
        }

        let trait_type_arguments = if let Some(type_arguments) = type_arguments {
            let mut type_argument_ids =
                TypeId::SmallVec::with_capacity(type_arguments.child_count());

            for type_argument in type_arguments.children() {
                let type_argument_id = self.get_explicit_type(type_argument)?;

                type_argument_ids.push(type_argument_id);
            }

            self.resolver.types.add_type_members(type_argument_ids)
        } else {
            TypeMembers::default()
        };

        self.current_self_type_id = previous_self_type_id;

        let impl_scope_id = self.current_scope_id;

        self.exit_scope();

        if type_parameters_scope_id != ScopeId::NONE {
            self.exit_scope();
        }

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
        let impl_declaration_id = self.add_scoped_declaration(Declaration {
            symbol_id: impl_symbol_id,
            definition,
            scope_id: self.current_scope_id,
            syntax: Some((reader.position(), reader.id)),
        });

        self.resolver
            .add_declaration_binding(reader.id, impl_declaration_id);
        self.resolver.add_scope_binding(body.id, impl_scope_id);

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

        let trait_name_str = self.source.get_content(&name.position())?;
        let trait_symbol_id = self.resolver.symbols.add_symbol(trait_name_str);
        let trait_declaration_id =
            self.reserve_scoped_declaration(trait_symbol_id, Some((reader.position(), reader.id)));

        let type_parameters_scope_id = if let Some(type_parameters) = type_parameters {
            self.bind_type_parameters(type_parameters)?
        } else {
            ScopeId::NONE
        };

        let supertraits_scope_id = if let Some(supertraits) = supertraits {
            self.enter_scope(ScopeKind::TypeTraitOrImpl);

            for supertrait in supertraits.children() {
                let supertrait_declaration_id = self.bind_path(supertrait)?;
                let supertrait_declaration = self
                    .resolver
                    .declarations
                    .get_declaration(supertrait_declaration_id)?;
                let supertrait_symbol_id = supertrait_declaration.symbol_id;

                self.scoped_declarations
                    .push((supertrait_symbol_id, supertrait_declaration_id));
            }

            let supertraits_scope_id = self.current_scope_id;
            self.exit_scope();

            supertraits_scope_id
        } else {
            ScopeId::NONE
        };

        self.enter_scope(ScopeKind::TypeTraitOrImpl);

        let self_symbol_id = self.resolver.symbols.add_self_symbol();
        let self_type_declaration_id = self.add_scoped_declaration(Declaration {
            symbol_id: self_symbol_id,
            definition: Definition::TypeParameter,
            scope_id: self.current_scope_id,
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
                    self.bind_type_item(child)?;

                    let TypeItem {
                        public: _,
                        name,
                        type_parameters: _,
                        aliased_type,
                    } = child.as_component()?;

                    let aliased_type_id = aliased_type
                        .map(|aliased_type| self.get_explicit_type(aliased_type))
                        .transpose()?;
                    let type_name_str = self.source.get_content(&name.position())?;
                    let type_symbol_id = self.resolver.symbols.add_symbol(type_name_str);
                    let type_declaration_id = self.add_scoped_declaration(Declaration {
                        symbol_id: type_symbol_id,
                        definition: Definition::TraitAssociatedType {
                            parent: trait_declaration_id,
                            type_parameters: ScopeId::NONE,
                            default_aliased_type_id: aliased_type_id,
                        },
                        scope_id: self.current_scope_id,
                        syntax: Some((child.position(), child.id)),
                    });

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

                    let const_name_str = self.source.get_content(&name.position())?;
                    let const_symbol_id = self.resolver.symbols.add_symbol(const_name_str);
                    let type_id = self.get_explicit_type(type_notation)?;
                    let const_declaration_id = self.add_scoped_declaration(Declaration {
                        symbol_id: const_symbol_id,
                        definition: Definition::InherentAssociatedConstant {
                            public: false,
                            parent: trait_declaration_id,
                            type_id,
                        },
                        scope_id: self.current_scope_id,
                        syntax: Some((child.position(), child.id)),
                    });

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

        let declarations_scope_id = self.current_scope_id;

        self.exit_scope();

        if type_parameters_scope_id != ScopeId::NONE {
            self.exit_scope();
        }

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
        self.resolver
            .add_scope_binding(body.id, declarations_scope_id);

        Ok(())
    }

    fn bind_statement(&mut self, reader: SyntaxReader) -> Result<(), CompileError> {
        match reader.node.kind {
            SyntaxKind::LetStatement => self.bind_let_statement(reader),
            SyntaxKind::ExpressionStatement => self.bind_expression_statement(reader),
            _ => Err(CompileError::UnexpectedSyntax {
                expected: &[SyntaxKind::LetStatement, SyntaxKind::ExpressionStatement],
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

        let identifier = self.source.get_content(&name.position())?;
        let symbol_id = self.resolver.symbols.add_symbol(identifier);
        let type_id = if let Some(type_notation) = type_notation {
            self.get_explicit_type(type_notation)?
        } else {
            self.resolver.types.create_inferred_type(None)
        };
        let shadowed = self
            .find_declaration_in_scope(symbol_id, self.current_scope_id)
            .map(|(declaration_id, _)| declaration_id);
        let declaration_id = self.add_scoped_declaration(Declaration {
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

    fn bind_expression_statement(&mut self, reader: SyntaxReader) -> Result<(), CompileError> {
        let ExpressionStatement { expression } = reader.as_component()?;

        self.bind_expression(expression)?;

        Ok(())
    }

    fn bind_expression(&mut self, reader: SyntaxReader) -> Result<(), CompileError> {
        match reader.node.kind {
            SyntaxKind::AdditionAssignmentExpression => self.bind_math_expression(reader),
            SyntaxKind::AssignmentExpression => self.bind_assignment_expression(reader),
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
            _ => Err(CompileError::UnexpectedSyntax {
                expected: &[
                    SyntaxKind::AssignmentExpression,
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
        if let Some(declaration_id) = search_path_segments(self, reader)? {
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
            let field_name_str = self.source.get_content(&field_name.position())?;
            let field_symbol_id = self.resolver.symbols.add_symbol(field_name_str);

            if let Some((field_declaration_id, _)) =
                self.find_declaration_direct(fields_scope_id, field_symbol_id)
            {
                self.resolver
                    .add_declaration_binding(field_name.id, field_declaration_id);
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
                match self.bind_statement(child) {
                    Ok(_) => {}
                    Err(error) => self.errors.push(ErrorKind::Compile(error)),
                }
            } else {
                self.bind_expression(child)?;
            }
        }

        self.current_scope_id = parent_scope_id;

        self.resolver.add_scope_binding(reader.id, block_scope_id);

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
                SyntaxKind::BlockExpression => self.bind_block_expression(else_branch, ())?,
                SyntaxKind::IfExpression => self.bind_if_expression(else_branch, ())?,
                _ => {
                    return Err(CompileError::UnexpectedSyntaxKind {
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

        let operand_declaration_id = *self
            .resolver
            .get_declaration_binding(&struct_expression.id)?;
        let operand_declaration = self
            .resolver
            .declarations
            .get_declaration(operand_declaration_id)?;

        let type_id = match operand_declaration.definition {
            Definition::Local { type_id, .. } | Definition::Field { type_id, .. } => type_id,
            Definition::Function { return_type_id, .. } => return_type_id,
            _ => {
                return Err(CompileError::ExpectedValue {
                    file_id: struct_expression.file_id(),
                    syntax_id: struct_expression.id,
                });
            }
        };

        let operand_type = *self.resolver.types.get_type(type_id)?;

        let field_name_str = self.source.get_content(&field_name.position())?;
        let field_symbol_id = self.resolver.symbols.add_symbol(field_name_str);

        let field_declaration_id = match operand_type {
            Type::Algebraic { declaration_id, .. } => {
                let struct_declaration =
                    self.resolver.declarations.get_declaration(declaration_id)?;

                let fields = match struct_declaration.definition {
                    Definition::StructType { fields, .. } | Definition::Variant { fields, .. } => {
                        fields
                    }
                    _ => {
                        return Err(CompileError::CannotAccessField {
                            type_id,
                            position: struct_expression.position(),
                        });
                    }
                };

                let field_entries = self.resolver.scopes.get_namespace_entries(fields);

                let mut found_field_declaration_id = None;

                for &(entry_symbol_id, field_declaration_id) in field_entries {
                    if entry_symbol_id == field_symbol_id {
                        found_field_declaration_id = Some(field_declaration_id);

                        break;
                    }
                }

                match found_field_declaration_id {
                    Some(id) => id,
                    None => {
                        let (member_id, _) =
                            search_impl_member(self, declaration_id, field_symbol_id, &field_name)?;

                        member_id
                    }
                }
            }
            Type::Generic { declaration_id } => {
                let generic_declaration =
                    self.resolver.declarations.get_declaration(declaration_id)?;
                let scope_id = generic_declaration.scope_id;

                match self.resolver.declarations.find_declaration(
                    field_symbol_id,
                    scope_id,
                    Visibility::Module,
                ) {
                    Some((member_id, _)) => member_id,
                    None => {
                        return Err(CompileError::CannotAccessField {
                            type_id,
                            position: struct_expression.position(),
                        });
                    }
                }
            }
            _ => {
                return Err(CompileError::CannotAccessField {
                    type_id,
                    position: struct_expression.position(),
                });
            }
        };

        self.resolver
            .add_declaration_binding(field_name.id, field_declaration_id);
        self.resolver
            .add_declaration_binding(reader.id, field_declaration_id);

        Ok(())
    }

    fn get_explicit_type(&mut self, reader: SyntaxReader) -> Result<TypeId, CompileError> {
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

                let element_type_id = self.get_explicit_type(element_type)?;
                let length_str = self.source.get_content(&length.position())?;
                let length = create_usize_from_decimal(length_str)?;

                Ok(self.resolver.types.add_type(Type::Array {
                    element_type_id,
                    length,
                }))
            }
            SyntaxKind::SliceType => {
                let element_type = reader.single_child()?;

                let element_type_id = self.get_explicit_type(element_type)?;
                let slice_symbol_id = self.resolver.symbols.add_symbol("[]");
                let declaration_id = self.add_scoped_declaration(Declaration {
                    symbol_id: slice_symbol_id,
                    definition: Definition::TypeParameter,
                    scope_id: self.current_scope_id,
                    syntax: Some((reader.position(), reader.id)),
                });

                Ok(self.resolver.types.add_type(Type::Slice {
                    declaration_id,
                    element_type_id,
                }))
            }
            SyntaxKind::TupleType => {
                let element_type_ids = reader
                    .children()
                    .map(|element_type| self.get_explicit_type(element_type))
                    .try_collect::<SmallVec<[TypeId; 4]>>()?;
                let element_type_ids = self.resolver.types.add_type_members(element_type_ids);

                Ok(self
                    .resolver
                    .types
                    .add_type(Type::Tuple { element_type_ids }))
            }
            SyntaxKind::FunctionType => {
                let FunctionType {
                    value_parameter_types,
                    return_type,
                } = reader.as_component()?;

                let value_parameter_ids = value_parameter_types
                    .children()
                    .map(|parameter_type| self.get_explicit_type(parameter_type))
                    .try_collect::<SmallVec<[TypeId; 4]>>()?;
                let value_parameters = self.resolver.types.add_type_members(value_parameter_ids);
                let return_type_id = if let Some(return_type) = return_type {
                    self.get_explicit_type(return_type)?
                } else {
                    TypeId::UNIT
                };

                Ok(self.resolver.types.add_type(Type::Function {
                    value_parameters,
                    return_type_id,
                }))
            }
            SyntaxKind::TypePath => {
                if let Some(declaration_id) = search_path_segments(self, reader)? {
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
                        _ => {
                            return Err(CompileError::ExpectedTypeDeclaration(declaration_id));
                        }
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
                    SyntaxKind::SliceType,
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
        debug!("Visiting path");
        debug_assert_eq!(path.node.kind, SyntaxKind::Path);

        let declaration_id = search_path_segments(self, path)?.ok_or_else(|| {
            let segment_str = self
                .source
                .get_content(&path.position())
                .unwrap_or("unknown");
            let symbol_id = self.resolver.symbols.add_symbol(segment_str);

            CompileError::Undeclared {
                symbol_id,
                usage_position: path.position(),
            }
        })?;

        self.resolver
            .add_declaration_binding(path.id, declaration_id);

        Ok(declaration_id)
    }

    fn bind_simple_path(
        &mut self,
        simple_path: SyntaxReader,
    ) -> Result<DeclarationId, CompileError> {
        debug!("Visiting simple path");
        debug_assert_eq!(simple_path.node.kind, SyntaxKind::SimplePath);

        let identifier = self.source.get_content(&simple_path.position())?;
        let symbol_id = self.resolver.symbols.add_symbol(identifier);
        let (declaration_id, _) = self
            .find_declaration_in_scope(symbol_id, self.current_scope_id)
            .ok_or(CompileError::Undeclared {
                symbol_id,
                usage_position: simple_path.position(),
            })?;

        self.resolver
            .add_declaration_binding(simple_path.id, declaration_id);

        Ok(declaration_id)
    }
}

fn search_path_segments<'a>(
    binder: &mut DeclarationBinder<'a>,
    path_expression: SyntaxReader,
) -> Result<Option<DeclarationId>, CompileError> {
    let file = binder.source.get_code(path_expression.file_id())?;

    let mut direct_scope: Option<ScopeId> = None;
    let mut parent_declaration_id = None;
    let mut impl_member_scope: Option<DeclarationId> = None;
    let mut current_declaration_id = None;

    for segment in path_expression.children() {
        let segment_str = file.get_str(segment.node.span)?;
        let symbol_id = binder.resolver.symbols.add_symbol(segment_str);

        let (next_declaration_id, next_definition) = if let Some(type_declaration_id) =
            impl_member_scope.take()
        {
            search_impl_member(binder, type_declaration_id, symbol_id, &segment)?
        } else if let Some(scope_id) = direct_scope {
            if let Some((id, declaration)) = binder.find_declaration_direct(scope_id, symbol_id) {
                (id, declaration.definition)
            } else {
                return Ok(None);
            }
        } else if let Some((id, declaration)) =
            binder.find_declaration_in_scope(symbol_id, binder.current_scope_id)
        {
            (id, declaration.definition)
        } else {
            return Ok(None);
        };

        match next_definition {
            Definition::Module { inner_scope_id, .. } => {
                direct_scope = Some(inner_scope_id);
            }
            Definition::StructType { .. } | Definition::EnumType { .. } => {
                impl_member_scope = Some(next_declaration_id);
            }
            Definition::Field {
                parent_struct: next_parent_id,
                ..
            }
            | Definition::Variant {
                enum_declaration_id: next_parent_id,
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

        current_declaration_id = Some(next_declaration_id);
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
                let member_entries = binder.resolver.scopes.get_namespace_entries(declarations);

                for &(_, member_id) in member_entries {
                    let member = binder.resolver.declarations.get_declaration(member_id)?;

                    if member.symbol_id == symbol_id {
                        return Ok((member_id, member.definition));
                    }
                }

                if let Definition::TraitImplementation {
                    trait_declaration_id,
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
                        let trait_member_entries = binder
                            .resolver
                            .scopes
                            .get_namespace_entries(trait_declarations);

                        for &(_, trait_member_id) in trait_member_entries {
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
            Definition::Variant {
                enum_declaration_id: parent_enum,
                ..
            } if parent_enum == type_declaration_id && declaration.symbol_id == symbol_id => {
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

fn lookup_symbol(
    scope_stack: &[ScopeFrame],
    scoped_declarations: &[(SymbolId, DeclarationId)],
    scopes: &Scopes,
    scope_id: ScopeId,
    symbol_id: SymbolId,
) -> Option<DeclarationId> {
    for (i, frame) in scope_stack.iter().enumerate() {
        if frame.scope_id == scope_id {
            let start = frame.type_entries_start as usize;
            let end = scope_stack
                .get(i + 1)
                .map(|next| next.type_entries_start as usize)
                .unwrap_or(scoped_declarations.len());

            for &(entry_symbol_id, declaration_id) in scoped_declarations[start..end].iter().rev() {
                if entry_symbol_id == symbol_id {
                    return Some(declaration_id);
                }
            }

            return None;
        }
    }

    scopes.find_in_namespace(scope_id, symbol_id)
}

fn is_barrier(scope_kind: ScopeKind, visibility: Visibility) -> bool {
    match (scope_kind, visibility) {
        (ScopeKind::Block, _) => false,
        (ScopeKind::Function, Visibility::Block) => true,
        (ScopeKind::Function, Visibility::Module) | (ScopeKind::Function, Visibility::Type) => {
            false
        }
        (ScopeKind::Closure, _) => false,
        (ScopeKind::Module, Visibility::Block) | (ScopeKind::Module, Visibility::Type) => true,
        (ScopeKind::Module, Visibility::Module) => false,
        (ScopeKind::TypeTraitOrImpl, Visibility::Block) => true,
        (ScopeKind::TypeTraitOrImpl, Visibility::Module)
        | (ScopeKind::TypeTraitOrImpl, Visibility::Type) => false,
        (ScopeKind::Associated, Visibility::Block) => true,
        (ScopeKind::Associated, Visibility::Module) | (ScopeKind::Associated, Visibility::Type) => {
            false
        }
        (ScopeKind::Constant, Visibility::Block) | (ScopeKind::Constant, Visibility::Type) => true,
        (ScopeKind::Constant, Visibility::Module) => false,
        (ScopeKind::TypeParameters, _) => false,
        (ScopeKind::ValueParameters, _) => false,
    }
}
