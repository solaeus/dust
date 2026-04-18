use crate::compiler::resolver::{declarations::DeclarationId, symbols::SymbolId};

#[derive(Debug, Default)]
pub struct Scopes {
    scopes: Vec<Scope>,
    namespace: Vec<(SymbolId, DeclarationId)>,
}

impl Scopes {
    pub fn new() -> Self {
        Self {
            scopes: Vec::new(),
            namespace: Vec::new(),
        }
    }

    pub fn enter_scope(&mut self, kind: ScopeKind, parent: ScopeId) -> ScopeId {
        let id = ScopeId(self.scopes.len() as u32);

        self.scopes.push(Scope {
            kind,
            parent,
            namespace_range: (0, 0),
        });

        id
    }

    pub fn exit_scope<T>(&mut self, id: ScopeId, entries: T)
    where
        T: IntoIterator<Item = (SymbolId, DeclarationId)>,
    {
        let scope = &mut self.scopes[id.0 as usize];

        scope.namespace_range.0 = self.namespace.len() as u32;

        self.namespace.extend(entries);

        scope.namespace_range.1 = self.namespace.len() as u32;
    }

    pub fn get_scope(&self, id: ScopeId) -> &Scope {
        &self.scopes[id.0 as usize]
    }
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Scope {
    kind: ScopeKind,
    pub parent: ScopeId,
    namespace_range: (u32, u32),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum ScopeKind {
    Block,
    Function,
    Closure,
    Module,
    TypeTraitOrImpl,
    Associated,
    Constant,
    TypeParameters,
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

pub struct ScopeFrame {
    pub scope_id: ScopeId,
    pub type_entries_start: u32,
}
