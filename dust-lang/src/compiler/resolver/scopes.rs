use std::num::NonZeroU32;

use crate::compiler::resolver::declarations::{DeclarationId, Definition};

#[derive(Debug, Default)]
pub struct Scopes {
    scopes: Vec<Scope>,
    scope_members: Vec<DeclarationId>,
    current_namespace: Vec<DeclarationId>,
}

impl Scopes {
    pub fn new() -> Self {
        Self {
            scopes: Vec::new(),
            scope_members: Vec::new(),
            current_namespace: Vec::new(),
        }
    }

    pub fn enter_scope(&mut self, kind: ScopeKind, parent: Option<ScopeId>) -> ScopeId {
        let id = ScopeId::from_index(self.scopes.len());
        let namespace_start = self.current_namespace.len() as u32;

        self.scopes.push(Scope {
            kind,
            parent,
            members_range: (namespace_start, u32::MAX),
        });

        id
    }

    pub fn add_to_current_namespace(&mut self, declaration_id: DeclarationId) {
        self.current_namespace.push(declaration_id);
    }

    pub fn exit_scope(&mut self, id: ScopeId) -> Option<ScopeId> {
        let scope = &mut self.scopes[id.index()];
        let namespace_start = scope.members_range.0 as usize;

        scope.members_range.0 = self.scope_members.len() as u32;

        self.scope_members
            .extend(self.current_namespace.drain(namespace_start..));

        scope.members_range.1 = self.scope_members.len() as u32;

        scope.parent
    }

    pub fn get_members(&self, id: ScopeId) -> &[DeclarationId] {
        let scope = &self.scopes[id.index()];
        let start = scope.members_range.0 as usize;
        let end = scope.members_range.1 as usize;

        &self.scope_members[start..end]
    }

    pub fn get_scope(&self, id: ScopeId) -> &Scope {
        &self.scopes[id.index()]
    }

    #[cfg(test)]
    pub fn iter(&self) -> impl Iterator<Item = (ScopeId, &Scope)> + '_ {
        self.scopes.iter().enumerate().map(|(index, scope)| {
            let id = ScopeId::from_index(index);

            (id, scope)
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Scope {
    pub kind: ScopeKind,
    pub parent: Option<ScopeId>,
    members_range: (u32, u32),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum ScopeKind {
    Module,
    Item,
    Associated,
    Block,
    Closure,
    Constant,
    Members,
}

impl ScopeKind {
    pub fn is_barrier(self, definition: &Definition) -> bool {
        matches!(
            (self, definition),
            (
                ScopeKind::Item | ScopeKind::Associated | ScopeKind::Constant,
                Definition::Local { .. } | Definition::Field { .. },
            ) | (
                ScopeKind::Item | ScopeKind::Constant,
                Definition::TypeParameter
            )
        )
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ScopeId(NonZeroU32);

impl ScopeId {
    #[expect(clippy::disallowed_methods)]
    pub const CORE: Self = ScopeId(NonZeroU32::new(1).unwrap());

    fn from_index(index: usize) -> Self {
        ScopeId(unsafe { NonZeroU32::new_unchecked((index + 1) as u32) })
    }

    fn index(self) -> usize {
        (self.0.get() - 1) as usize
    }
}
