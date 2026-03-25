pub mod declarations;
pub mod error;
pub mod scopes;
pub mod symbols;
pub mod types;

use std::collections::{HashMap, HashSet};

use rustc_hash::FxBuildHasher;
use smallvec::SmallVec;

use crate::{
    compiler::error::CompileError,
    dust_type::DustType,
    resolver::{
        declarations::{
            Declaration, DeclarationId, DeclarationMembers, Declarations, Definition, Visibility,
        },
        error::ResolverError,
        scopes::{ScopeId, Scopes},
        symbols::{SymbolId, Symbols},
        types::{
            FloatType, SignedIntegerType, Type, TypeId, TypeMembers, Types, UnsignedIntegerType,
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

    scope_search: HashSet<ScopeId, FxBuildHasher>,

    declaration_bindings: HashMap<SyntaxId, DeclarationId, FxBuildHasher>,
    scope_bindings: HashMap<SyntaxId, ScopeId, FxBuildHasher>,
    type_bindings: HashMap<SyntaxId, TypeId, FxBuildHasher>,
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
        _id: TypeId,
        _source: &Source,
    ) -> Result<DustType, CompileError> {
        todo!()
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

    pub fn declaration_display_iterator<'a>(
        &'a self,
    ) -> impl Iterator<Item = Result<String, CompileError>> + 'a {
        self.declarations.iter().map(|(_id, _declaration)| todo!())
    }
}

impl Default for Resolver {
    fn default() -> Self {
        Self::new()
    }
}
