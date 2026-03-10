pub mod declaration_graph;
pub mod error;
pub mod scope_graph;
pub mod symbol_table;
pub mod type_graph;

use std::collections::{HashMap, HashSet};

use rustc_hash::FxBuildHasher;
use smallvec::SmallVec;

use crate::{
    compiler::error::CompileError,
    dust_type::{DustFunctionType, DustStructType, DustType},
    native_function::NativeFunction,
    resolver::{
        declaration_graph::{
            Declaration, DeclarationGraph, DeclarationId, DeclarationKind, DeclarationMembers,
            ModuleKind, Visibility,
        },
        error::ResolverError,
        scope_graph::{Scope, ScopeGraph, ScopeId, ScopeKind},
        symbol_table::{SymbolId, SymbolTable},
        type_graph::{TypeGraph, TypeId, TypeMembers, TypeNode},
    },
    source::Source,
    syntax::{SyntaxId, SyntaxReader},
};

#[derive(Debug)]
pub struct Resolver {
    pub symbols: SymbolTable,
    pub declarations: DeclarationGraph,
    pub scopes: ScopeGraph,
    pub types: TypeGraph,

    scope_search: HashSet<ScopeId, FxBuildHasher>,

    declaration_bindings: HashMap<SyntaxId, DeclarationId, FxBuildHasher>,
    scope_bindings: HashMap<SyntaxId, ScopeId, FxBuildHasher>,
    type_bindings: HashMap<SyntaxId, TypeId, FxBuildHasher>,
}

impl Resolver {
    pub fn new() -> Self {
        let mut resolver = Self {
            symbols: SymbolTable::new(),
            declarations: DeclarationGraph::new(),
            scopes: ScopeGraph::new(),
            types: TypeGraph::new(),
            scope_search: HashSet::default(),
            declaration_bindings: HashMap::default(),
            scope_bindings: HashMap::default(),
            type_bindings: HashMap::default(),
        };

        resolver.add_core();

        resolver
    }

    fn add_core(&mut self) {
        let mut core_items = SmallVec::<[DeclarationId; 4]>::with_capacity(4);

        let option_declaration_id = self.declarations.next_declaration_id();
        let none_declaration_id = option_declaration_id.offset(1);
        let some_field_declaration_id = option_declaration_id.offset(2);
        let some_declaration_id = option_declaration_id.offset(3);

        let option_symbol = self.symbols.add_symbol("Option");
        let none_symbol = self.symbols.add_symbol("None");
        let some_field_symbol = self.symbols.add_symbol("0");
        let some_symbol = self.symbols.add_symbol("Some");

        let _option_declaration_id = {
            let members = self
                .declarations
                .add_declaration_members(&[none_declaration_id, some_declaration_id]);

            self.declarations.add_declaration(Declaration {
                symbol_id: option_symbol,
                syntax: None,
                kind: DeclarationKind::Type {
                    parent: None,
                    type_parameters: DeclarationMembers::default(),
                    members,
                },
                scope_id: ScopeId::CORE,
                public: true,
            })
        };
        let _none_declaration_id = self.declarations.add_declaration(Declaration {
            symbol_id: none_symbol,
            syntax: None,
            kind: DeclarationKind::Type {
                parent: Some(option_declaration_id),
                type_parameters: DeclarationMembers::default(),
                members: DeclarationMembers::default(),
            },
            scope_id: ScopeId::CORE,
            public: true,
        });
        let _some_field_declaration_id = self.declarations.add_declaration(Declaration {
            symbol_id: some_field_symbol,
            syntax: None,
            kind: DeclarationKind::Type {
                parent: Some(some_declaration_id),
                type_parameters: DeclarationMembers::default(),
                members: DeclarationMembers::default(),
            },
            scope_id: ScopeId::CORE,
            public: true,
        });
        let _some_declaration_id = {
            let members = self
                .declarations
                .add_declaration_members(&[some_field_declaration_id]);

            self.declarations.add_declaration(Declaration {
                symbol_id: some_symbol,
                syntax: None,
                kind: DeclarationKind::Type {
                    parent: Some(option_declaration_id),
                    type_parameters: DeclarationMembers::default(),
                    members,
                },
                scope_id: ScopeId::CORE,
                public: true,
            })
        };

        let vec_declaration_id = self.declarations.next_declaration_id();
        let element_type_parameter_declaration_id = vec_declaration_id.offset(1);
        let with_capacity_declaration_id = vec_declaration_id.offset(2);

        let vec_symbol_id = self.symbols.add_symbol("Vec");
        let element_type_parameter_symbol = self.symbols.add_symbol("T");
        let with_capacity_symbol_id = self.symbols.add_symbol("with_capacity");

        let _vec_declaration_id = {
            let type_parameters = self
                .declarations
                .add_declaration_members(&[element_type_parameter_declaration_id]);
            let members = self
                .declarations
                .add_declaration_members(&[with_capacity_declaration_id]);

            self.declarations.add_declaration(Declaration {
                symbol_id: vec_symbol_id,
                syntax: None,
                kind: DeclarationKind::Type {
                    parent: None,
                    type_parameters,
                    members,
                },
                scope_id: ScopeId::CORE,
                public: true,
            })
        };

        let _element_type_parameter_declaration_id =
            self.declarations.add_declaration(Declaration {
                symbol_id: element_type_parameter_symbol,
                syntax: None,
                kind: DeclarationKind::Type {
                    parent: Some(vec_declaration_id),
                    type_parameters: DeclarationMembers::default(),
                    members: DeclarationMembers::default(),
                },
                scope_id: ScopeId::CORE,
                public: false,
            });
        let element_type_parameter_type_id = self.types.create_inferred_type();

        self.declarations.set_declaration_type(
            element_type_parameter_declaration_id,
            element_type_parameter_type_id,
        );

        let _with_capacity_declaration_id = self.declarations.add_declaration(Declaration {
            symbol_id: with_capacity_symbol_id,
            syntax: None,
            kind: DeclarationKind::NativeFunction(NativeFunction::VEC_WITH_CAPACITY),
            scope_id: ScopeId::CORE,
            public: true,
        });

        core_items.push(option_declaration_id);
        core_items.push(none_declaration_id);
        core_items.push(some_declaration_id);
        core_items.push(vec_declaration_id);

        debug_assert_eq!(option_declaration_id, _option_declaration_id);
        debug_assert_eq!(none_declaration_id, _none_declaration_id);
        debug_assert_eq!(some_field_declaration_id, _some_field_declaration_id);
        debug_assert_eq!(some_declaration_id, _some_declaration_id);
        debug_assert_eq!(vec_declaration_id, _vec_declaration_id);
        debug_assert_eq!(
            element_type_parameter_declaration_id,
            _element_type_parameter_declaration_id
        );
        debug_assert_eq!(with_capacity_declaration_id, _with_capacity_declaration_id);

        let core_scope_id = self.scopes.add_scope(Scope {
            kind: ScopeKind::Module,
            parent: ScopeId::NONE,
            modules: SmallVec::new(),
            imports: core_items,
        });

        debug_assert_eq!(core_scope_id, ScopeId::CORE);

        let core_symbol_id = self.symbols.add_symbol("core");

        self.declarations.add_declaration(Declaration {
            symbol_id: core_symbol_id,
            syntax: None,
            kind: DeclarationKind::Module {
                kind: ModuleKind::Inline,
                inner_scope_id: core_scope_id,
            },
            scope_id: ScopeId::NONE,
            public: true,
        });
    }

    pub fn add_declaration_binding(&mut self, syntax_id: SyntaxId, declaration_id: DeclarationId) {
        self.declaration_bindings.insert(syntax_id, declaration_id);
    }

    pub fn get_declaration_binding(
        &self,
        syntax_id: &SyntaxId,
    ) -> Result<&DeclarationId, ResolverError> {
        self.declaration_bindings
            .get(syntax_id)
            .ok_or(ResolverError::MissingDeclarationBinding(*syntax_id))
    }

    pub fn add_scope_binding(&mut self, syntax_id: SyntaxId, scope_id: ScopeId) {
        self.scope_bindings.insert(syntax_id, scope_id);
    }

    pub fn get_scope_binding(&self, syntax_id: &SyntaxId) -> Result<&ScopeId, ResolverError> {
        self.scope_bindings
            .get(syntax_id)
            .ok_or(ResolverError::MissingScopeBinding(*syntax_id))
    }

    pub fn add_type_binding(&mut self, syntax_id: SyntaxId, type_id: TypeId) {
        self.type_bindings.insert(syntax_id, type_id);
    }

    pub fn get_type_binding(&self, syntax_id: &SyntaxId) -> Result<&TypeId, ResolverError> {
        self.type_bindings
            .get(syntax_id)
            .ok_or(ResolverError::MissingTypeBinding(*syntax_id))
    }

    pub fn get_byte_size(
        &self,
        type_id: TypeId,
        node: &SyntaxReader,
    ) -> Result<usize, CompileError> {
        let type_node = self.types.get_type(type_id)?;

        match type_node {
            TypeNode::Boolean | TypeNode::Character | TypeNode::U8 | TypeNode::I8 => Ok(1),
            TypeNode::U16 | TypeNode::I16 | TypeNode::Function { .. } => Ok(2),
            TypeNode::U32 | TypeNode::I32 | TypeNode::F32 => Ok(4),
            TypeNode::U64 | TypeNode::I64 | TypeNode::F64 => Ok(8),
            TypeNode::U128 | TypeNode::I128 => Ok(16),
            TypeNode::Vec { .. } | TypeNode::String | TypeNode::List { .. } => {
                Ok(std::mem::size_of::<usize>())
            }
            TypeNode::Struct { declaration_id, .. } => {
                let struct_declaration = self.declarations.get_declaration(*declaration_id)?;

                let DeclarationKind::Type { members, .. } = struct_declaration.kind else {
                    return Err(CompileError::Resolver(
                        ResolverError::ExpectedTypeDeclaration(*declaration_id),
                    ));
                };
                let field_ids = self.declarations.get_declaration_members(members)?;
                let mut size = 0;

                for field_id in field_ids {
                    let field_type_id = self.declarations.get_declaration_type(field_id)?;
                    size += self.get_byte_size(*field_type_id, node)?;
                }

                Ok(size)
            }
            TypeNode::Inferred { resolved, .. } => {
                if let Some(resolved) = resolved {
                    self.get_byte_size(*resolved, node)
                } else {
                    Err(CompileError::CannotInferType {
                        type_id,
                        position: None,
                    })
                }
            }
            TypeNode::Unit | TypeNode::Enum { .. } => Err(CompileError::ExpectedValue {
                node_kind: node.kind(),
                position: node.position(),
            }),
        }
    }

    pub fn get_register_size(
        &self,
        type_id: TypeId,
        node: &SyntaxReader,
    ) -> Result<usize, CompileError> {
        self.get_byte_size(type_id, node)
            .map(|byte_size| byte_size.div_ceil(4))
    }

    pub fn find_declaration_in_scope(
        &mut self,
        symbol_id: SymbolId,
        target_scope_id: ScopeId,
        visibility: Visibility,
        path_segment: &SyntaxReader,
    ) -> Result<(DeclarationId, &Declaration), CompileError> {
        let mut current_scope_id = target_scope_id;

        loop {
            if current_scope_id == ScopeId::NONE || !self.scope_search.insert(current_scope_id) {
                break;
            }

            if let Some((declaration_id, declaration)) =
                self.declarations
                    .find_declaration(symbol_id, current_scope_id, visibility)
            {
                self.scope_search.clear();

                return Ok((declaration_id, declaration));
            }

            let current_scope = self.scopes.get_scope(current_scope_id)?;

            for module_scope_id in &current_scope.modules {
                if let Some((declaration_id, declaration)) =
                    self.declarations
                        .find_declaration(symbol_id, *module_scope_id, visibility)
                {
                    self.scope_search.clear();

                    return Ok((declaration_id, declaration));
                }
            }

            for import_declaration_id in &current_scope.imports {
                let import_declaration =
                    self.declarations.get_declaration(*import_declaration_id)?;

                if import_declaration.symbol_id == symbol_id {
                    self.scope_search.clear();

                    return Ok((*import_declaration_id, import_declaration));
                }
            }

            current_scope_id = current_scope.parent;
        }

        self.scope_search.clear();

        Err(CompileError::Undeclared {
            symbol_id,
            usage_position: path_segment.position(),
        })
    }

    pub fn add_external_type(&mut self, new_type: &DustType) -> TypeId {
        let node = match new_type {
            DustType::Unit => TypeNode::Unit,
            DustType::Boolean => TypeNode::Boolean,
            DustType::U8 => TypeNode::U8,
            DustType::I8 => TypeNode::I8,
            DustType::U16 => TypeNode::U16,
            DustType::I16 => TypeNode::I16,
            DustType::U32 => TypeNode::U32,
            DustType::I32 => TypeNode::I32,
            DustType::U64 => TypeNode::U64,
            DustType::I64 => TypeNode::I64,
            DustType::U128 => TypeNode::U128,
            DustType::I128 => TypeNode::I128,
            DustType::F32 => TypeNode::F32,
            DustType::F64 => TypeNode::F64,
            DustType::Character => TypeNode::Character,
            DustType::Vec(element_type) => {
                let element_type = self.add_external_type(element_type);

                TypeNode::Vec {
                    element_type_id: element_type,
                }
            }
            DustType::String => TypeNode::String,
            DustType::List(element_type) => {
                let element_type = self.add_external_type(element_type);

                TypeNode::List {
                    element_type_id: element_type,
                }
            }
            DustType::Function(function_type) => {
                let mut type_parameters = SmallVec::<[DeclarationId; 4]>::with_capacity(
                    function_type.type_parameters.len(),
                );

                for type_parameter_name in &function_type.type_parameters {
                    let symbol = self.symbols.add_symbol(type_parameter_name);
                    let type_parameter_id = self.declarations.add_declaration(Declaration {
                        symbol_id: symbol,
                        kind: DeclarationKind::Type {
                            parent: None,
                            type_parameters: DeclarationMembers::default(),
                            members: DeclarationMembers::default(),
                        },
                        scope_id: ScopeId::NONE,
                        public: false,
                        syntax: None,
                    });
                    let type_parameter_type_id = self.types.create_inferred_type();

                    type_parameters.push(type_parameter_id);
                    self.declarations
                        .set_declaration_type(type_parameter_id, type_parameter_type_id);
                }

                let mut value_parameter_types: SmallVec<[TypeId; 8]> =
                    SmallVec::with_capacity(function_type.value_parameters.len());

                for r#type in &function_type.value_parameters {
                    value_parameter_types.push(self.add_external_type(r#type));
                }

                TypeNode::Function {
                    type_parameters: self.declarations.add_declaration_members(&type_parameters),
                    value_parameters: self.types.add_type_members(&value_parameter_types),
                    return_type_id: self.add_external_type(&function_type.return_type),
                }
            }
            DustType::Struct(struct_type) => {
                let DustStructType { name, fields } = struct_type.as_ref();

                let symbol = self.symbols.add_symbol(name);
                let struct_declaration_id = self.declarations.next_declaration_id();

                let mut field_declaration_ids =
                    SmallVec::<[DeclarationId; 8]>::with_capacity(fields.len());

                for (field_name, field_type) in fields {
                    let symbol = self.symbols.add_symbol(field_name);
                    let declaration_id = self.declarations.add_declaration(Declaration {
                        symbol_id: symbol,
                        kind: DeclarationKind::Type {
                            parent: Some(struct_declaration_id),
                            type_parameters: DeclarationMembers::default(),
                            members: DeclarationMembers::default(),
                        },
                        scope_id: ScopeId::NONE,
                        public: false,
                        syntax: None,
                    });
                    let type_id = self.add_external_type(field_type);

                    field_declaration_ids.push(declaration_id);
                    self.declarations
                        .set_declaration_type(declaration_id, type_id);
                }

                let members = self
                    .declarations
                    .add_declaration_members(&field_declaration_ids);
                let declared_id = self.declarations.add_declaration(Declaration {
                    symbol_id: symbol,
                    kind: DeclarationKind::Type {
                        parent: None,
                        type_parameters: DeclarationMembers::default(),
                        members,
                    },
                    scope_id: ScopeId::NONE,
                    public: false,
                    syntax: None,
                });

                debug_assert_eq!(declared_id, struct_declaration_id);

                TypeNode::Struct {
                    declaration_id: struct_declaration_id,
                    type_arguments: TypeMembers::default(),
                }
            }
        };

        self.types.add_type(node)
    }

    pub fn get_full_type(&self, id: TypeId, source: &Source) -> Result<DustType, CompileError> {
        let type_node = self.types.get_type(id)?;

        match type_node {
            TypeNode::Unit => Ok(DustType::Unit),
            TypeNode::Boolean => Ok(DustType::Boolean),
            TypeNode::U8 => Ok(DustType::U8),
            TypeNode::I8 => Ok(DustType::I8),
            TypeNode::U16 => Ok(DustType::U16),
            TypeNode::I16 => Ok(DustType::I16),
            TypeNode::U32 => Ok(DustType::U32),
            TypeNode::I32 => Ok(DustType::I32),
            TypeNode::U64 => Ok(DustType::U64),
            TypeNode::I64 => Ok(DustType::I64),
            TypeNode::U128 => Ok(DustType::U128),
            TypeNode::I128 => Ok(DustType::I128),
            TypeNode::F32 => Ok(DustType::F32),
            TypeNode::F64 => Ok(DustType::F64),
            TypeNode::Character => Ok(DustType::Character),
            TypeNode::Vec { element_type_id } => {
                let element_type = self.get_full_type(*element_type_id, source)?;

                Ok(DustType::Vec(Box::new(element_type)))
            }
            TypeNode::String => Ok(DustType::String),
            TypeNode::List { element_type_id } => {
                let element_type = self.get_full_type(*element_type_id, source)?;

                Ok(DustType::list(element_type))
            }
            TypeNode::Function {
                type_parameters,
                value_parameters,
                return_type_id,
            } => {
                let type_parameters = self
                    .get_declaration_member_names(*type_parameters)
                    .try_collect()?;
                let value_parameters = self
                    .get_type_members_as_full_types(*value_parameters, source)
                    .try_collect()?;
                let return_type = self.get_full_type(*return_type_id, source)?;

                Ok(DustType::Function(Box::new(DustFunctionType {
                    type_parameters,
                    value_parameters,
                    return_type,
                })))
            }
            TypeNode::Inferred { resolved, .. } => {
                if let Some(resolved) = resolved {
                    self.get_full_type(*resolved, source)
                } else {
                    Err(CompileError::CannotInferType {
                        type_id: id,
                        position: None,
                    })
                }
            }
            TypeNode::Struct { declaration_id, .. } => {
                let struct_declaration = self.declarations.get_declaration(*declaration_id)?;
                let struct_name = self
                    .symbols
                    .get_symbol(&struct_declaration.symbol_id)?
                    .to_string();

                let DeclarationKind::Type { members, .. } = struct_declaration.kind else {
                    return Err(CompileError::Resolver(ResolverError::MissingDeclaration(
                        *declaration_id,
                    )));
                };
                let field_ids = self.declarations.get_declaration_members(members)?;
                let mut field_types = Vec::with_capacity(field_ids.len());

                for field_id in field_ids {
                    let field_declaration = self.declarations.get_declaration(*field_id)?;
                    let field_name = self
                        .symbols
                        .get_symbol(&field_declaration.symbol_id)?
                        .to_string();

                    let field_type_id = self.declarations.get_declaration_type(field_id)?;
                    let field_type = self.get_full_type(*field_type_id, source)?;

                    field_types.push((field_name, field_type));
                }

                Ok(DustType::Struct(Box::new(DustStructType {
                    name: struct_name,
                    fields: field_types,
                })))
            }
            TypeNode::Enum { .. } => {
                todo!()
            }
        }
    }

    fn get_declaration_member_names(
        &self,
        members: DeclarationMembers,
    ) -> impl Iterator<Item = Result<String, CompileError>> {
        members.as_range().map(|member_index| {
            let declaration_id = self.declarations.get_declaration_member(member_index)?;
            let declaration = self.declarations.get_declaration(*declaration_id)?;
            let name = self.symbols.get_symbol(&declaration.symbol_id)?.to_string();

            Ok(name)
        })
    }

    fn get_type_members_as_full_types(
        &self,
        members: TypeMembers,
        source: &Source,
    ) -> impl Iterator<Item = Result<DustType, CompileError>> {
        members.as_range().map(|member_index| {
            let type_id = *self.types.get_type_member(member_index)?;

            self.get_full_type(type_id, source)
        })
    }

    pub fn declaration_display_iterator<'a>(
        &'a self,
    ) -> impl Iterator<Item = Result<String, CompileError>> + 'a {
        self.declarations.iter().map(|(id, declaration)| {
            let symbol = self.symbols.get_symbol(&declaration.symbol_id)?;
            let kind_str = match declaration.kind {
                DeclarationKind::Module { .. } => "module",
                DeclarationKind::Type { parent, .. } => {
                    if let Some(parent) = parent {
                        let parent_declaration = self.declarations.get_declaration(parent)?;
                        let parent_symbol =
                            self.symbols.get_symbol(&parent_declaration.symbol_id)?;

                        &format!("type ({}::{})", parent_symbol, symbol)
                    } else {
                        "type"
                    }
                }
                DeclarationKind::NativeFunction(_) => "native function",
                DeclarationKind::Function { .. } => "function",
                DeclarationKind::Local { .. } => "local",
            };

            Ok(format!("ID {}: {symbol} {kind_str}", id.inner()))
        })
    }
}

impl Default for Resolver {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolver_size() {
        assert_eq!(size_of::<Resolver>(), 0);
    }
}
