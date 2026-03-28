pub mod declarations;
pub mod error;
pub mod scopes;
pub mod symbols;
pub mod types;

use std::collections::{HashMap, HashSet, VecDeque};

use rustc_hash::FxBuildHasher;
use smallvec::SmallVec;

use crate::{
    compiler::error::CompileError,
    dust_type::DustType,
    prototype::PrototypeId,
    resolver::{
        declarations::{Declaration, DeclarationId, DeclarationMembers, Declarations, Visibility},
        error::ResolverError,
        scopes::{ScopeId, Scopes},
        symbols::{SymbolId, Symbols},
        types::{
            FloatType, InferredTypeConstraint, SignedIntegerType, Type, TypeId, TypeMembers, Types,
            UnsignedIntegerType,
        },
    },
    source::Source,
    syntax::{SyntaxId, reader::SyntaxReader},
};

#[derive(Debug)]
pub struct Resolver {
    pub symbols: Symbols,
    pub declarations: Declarations,
    pub scopes: Scopes,
    pub types: Types,
    pub type_parameter_map: HashMap<DeclarationId, TypeId>,
    pub compilation_queue: VecDeque<CompilationRequest>,

    scope_search: HashSet<ScopeId, FxBuildHasher>,

    declaration_bindings: HashMap<SyntaxId, DeclarationId, FxBuildHasher>,
    scope_bindings: HashMap<SyntaxId, ScopeId, FxBuildHasher>,
    type_bindings: HashMap<SyntaxId, TypeId, FxBuildHasher>,
    monomorphization_cache: HashMap<(DeclarationId, SmallVec<[TypeId; 4]>), PrototypeId>,
}

impl Resolver {
    pub fn new() -> Self {
        let resolver = Self {
            symbols: Symbols::new(),
            declarations: Declarations::new(),
            scopes: Scopes::new(),
            types: Types::new(),
            scope_search: HashSet::default(),
            declaration_bindings: HashMap::default(),
            scope_bindings: HashMap::default(),
            type_bindings: HashMap::default(),
            type_parameter_map: HashMap::new(),
            monomorphization_cache: HashMap::new(),
            compilation_queue: VecDeque::new(),
        };

        // resolver.add_core();

        resolver
    }

    fn add_core(&mut self) {
        todo!()
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

    pub fn set_type_parameter_to_argument(
        &mut self,
        declaration_id: DeclarationId,
        type_id: TypeId,
    ) {
        self.type_parameter_map.insert(declaration_id, type_id);
    }

    pub fn get_type_parameter_as_argument(
        &self,
        declaration_id: &DeclarationId,
    ) -> Option<&TypeId> {
        self.type_parameter_map.get(declaration_id)
    }

    pub fn get_cached_prototype(
        &self,
        cache_key: &(DeclarationId, SmallVec<[TypeId; 4]>),
    ) -> Option<PrototypeId> {
        self.monomorphization_cache.get(cache_key).copied()
    }

    pub fn cache_prototype(
        &mut self,
        cache_key: (DeclarationId, SmallVec<[TypeId; 4]>),
        prototype_id: PrototypeId,
    ) {
        self.monomorphization_cache.insert(cache_key, prototype_id);
    }

    pub fn resolve_type_through_map(&mut self, type_id: TypeId) -> Result<TypeId, ResolverError> {
        let resolved_type = *self.types.get_type(type_id)?;

        let start_type_id = if let Type::Generic { declaration_id } = resolved_type
            && let Some(&inferred_type_id) = self.type_parameter_map.get(&declaration_id)
        {
            inferred_type_id
        } else if matches!(resolved_type, Type::Inferred { .. }) {
            type_id
        } else {
            return Ok(type_id);
        };

        let mut current_type_id = start_type_id;

        loop {
            match *self.types.get_type(current_type_id)? {
                Type::Inferred {
                    resolved: Some(resolved),
                    ..
                } => {
                    current_type_id = resolved;
                }
                Type::Inferred {
                    inferred_id,
                    constraint: Some(constraint),
                    resolved: None,
                } => {
                    let default_type_id = match constraint {
                        InferredTypeConstraint::Integer => TypeId::I_32,
                        InferredTypeConstraint::Float => TypeId::F_64,
                    };

                    let node = self.types.get_type_mut(current_type_id)?;

                    *node = Type::Inferred {
                        inferred_id,
                        constraint: Some(constraint),
                        resolved: Some(default_type_id),
                    };

                    return Ok(default_type_id);
                }
                _ => return Ok(current_type_id),
            }
        }
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

    pub fn add_external_type(&mut self, _new_type: &DustType) -> TypeId {
        todo!()
    }

    pub fn get_external_type(
        &self,
        id: TypeId,
        _source: &Source,
    ) -> Result<DustType, CompileError> {
        let r#type = self.types.get_type(id)?;

        match r#type {
            Type::Never => Ok(DustType::Unit),
            Type::Boolean => Ok(DustType::Boolean),
            Type::Character => Ok(DustType::Character),
            Type::SignedInteger(si) => match si {
                SignedIntegerType::I8 => Ok(DustType::I8),
                SignedIntegerType::I16 => Ok(DustType::I16),
                SignedIntegerType::I32 => Ok(DustType::I32),
                SignedIntegerType::I64 => Ok(DustType::I64),
                SignedIntegerType::I128 => Ok(DustType::I128),
            },
            Type::UnsignedInteger(ui) => match ui {
                UnsignedIntegerType::U8 => Ok(DustType::U8),
                UnsignedIntegerType::U16 => Ok(DustType::U16),
                UnsignedIntegerType::U32 => Ok(DustType::U32),
                UnsignedIntegerType::U64 => Ok(DustType::U64),
                UnsignedIntegerType::U128 => Ok(DustType::U128),
            },
            Type::Float(ft) => match ft {
                FloatType::F32 => Ok(DustType::F32),
                FloatType::F64 => Ok(DustType::F64),
            },
            Type::Tuple { element_type_ids } => {
                let members = self.types.get_type_members(*element_type_ids)?;

                if members.is_empty() {
                    Ok(DustType::Unit)
                } else {
                    todo!()
                }
            }
            Type::Inferred {
                resolved: Some(resolved),
                ..
            } => self.get_external_type(*resolved, _source),
            _ => todo!("{type:?}"),
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

            self.get_external_type(type_id, source)
        })
    }

    pub fn definition_display_iterator<'a>(
        &'a self,
    ) -> impl Iterator<Item = Result<(&str, String), CompileError>> + 'a {
        self.declarations.iter().map(|(_, declaration)| {
            let symbol = self.symbols.get_symbol(&declaration.symbol_id)?;

            Ok((symbol, format!("{:#?}", declaration.definition)))
        })
    }
}

impl Default for Resolver {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug)]
pub struct CompilationRequest {
    pub declaration_id: DeclarationId,

    pub prototype_id: PrototypeId,
}
