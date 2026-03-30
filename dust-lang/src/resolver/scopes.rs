use smallvec::SmallVec;

use crate::resolver::{declarations::DeclarationId, error::ResolverError};

#[derive(Debug, Default)]
pub struct Scopes {
    scopes: Vec<Scope>,
}

impl Scopes {
    pub fn new() -> Self {
        Self { scopes: Vec::new() }
    }

    pub fn add_scope(&mut self, scope: Scope) -> ScopeId {
        let id = ScopeId(self.scopes.len() as u32);

        self.scopes.push(scope);

        id
    }

    pub fn get_scope(&self, id: ScopeId) -> Result<&Scope, ResolverError> {
        self.scopes
            .get(id.0 as usize)
            .ok_or(ResolverError::MissingScope(id))
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ScopeId(u32);

impl ScopeId {
    pub const NONE: Self = ScopeId(u32::MAX);
    pub const CORE: Self = ScopeId(0);

    pub fn inner(self) -> u32 {
        self.0
    }
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Scope {
    pub kind: ScopeKind,
    pub parent: ScopeId,
    pub modules: SmallVec<[ScopeId; 4]>,
    pub imports: SmallVec<[DeclarationId; 4]>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ScopeKind {
    Crate,
    Module,
    Function,
    Block,
}
