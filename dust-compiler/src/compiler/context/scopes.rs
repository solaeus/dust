use crate::compiler::context::declarations::{DeclarationId, Definition};

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

    pub fn scope_count(&self) -> usize {
        self.scopes.len()
    }

    pub fn enter_scope(&mut self, barrier: Barrier, parent: Option<ScopeId>) -> ScopeId {
        let id = ScopeId::from_index(self.scopes.len());
        let namespace_start = self.current_namespace.len() as u32;

        self.scopes.push(Scope {
            barrier,
            parent,
            members_range: (namespace_start, u32::MAX),
        });

        id
    }

    pub fn add_to_current_scope(&mut self, declaration_id: DeclarationId) {
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
        self.scopes
            .iter()
            .enumerate()
            .map(|(index, scope)| (ScopeId::from_index(index), scope))
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Scope {
    pub barrier: Barrier,
    pub parent: Option<ScopeId>,
    members_range: (u32, u32),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ScopeId(u32);

impl ScopeId {
    pub const CORE: Self = ScopeId(0);

    fn from_index(index: usize) -> Self {
        ScopeId(index as u32)
    }

    fn index(self) -> usize {
        self.0 as usize
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Barrier {
    Module,
    Item,
    Associated,
    Block,
    Closure,
    Constant,
    Members,
}

impl Barrier {
    pub fn blocks(self, definition: &Definition) -> bool {
        matches!(
            (self, definition),
            (
                Barrier::Item | Barrier::Associated | Barrier::Constant,
                Definition::Local { .. } | Definition::Field { .. },
            ) | (
                Barrier::Item | Barrier::Constant,
                Definition::TypeParameter { .. }
            )
        )
    }
}

#[derive(Default)]
pub struct BarrierTracker {
    item: bool,
    associated: bool,
    constant: bool,
}

impl BarrierTracker {
    pub fn should_block(&mut self, definition: &Definition) -> bool {
        let block_variable = (self.item || self.associated || self.constant)
            && matches!(
                definition,
                Definition::Local { .. } | Definition::Field { .. }
            );

        if block_variable {
            return true;
        }

        (self.item || self.constant) && matches!(definition, Definition::TypeParameter { .. })
    }

    pub fn add(&mut self, barrier: Barrier) {
        match barrier {
            Barrier::Item => self.item = true,
            Barrier::Associated => self.associated = true,
            Barrier::Constant => self.constant = true,
            _ => {}
        }
    }
}
