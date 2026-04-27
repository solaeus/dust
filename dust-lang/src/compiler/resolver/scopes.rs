use std::num::NonZeroU32;

use crate::compiler::resolver::{
    declarations::{DeclarationId, Definition},
    symbols::SymbolId,
};

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

    pub fn enter_scope(&mut self, kind: ScopeKind, parent: Option<ScopeId>) -> ScopeId {
        let id = ScopeId::from_index(self.scopes.len());

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
        let scope = &mut self.scopes[id.index()];

        scope.namespace_range.0 = self.namespace.len() as u32;

        self.namespace.extend(entries);

        scope.namespace_range.1 = self.namespace.len() as u32;
    }

    pub fn get_scope(&self, id: ScopeId) -> &Scope {
        &self.scopes[id.index()]
    }

    pub fn get_namespace_entries(&self, id: ScopeId) -> &[(SymbolId, DeclarationId)] {
        let scope = &self.scopes[id.index()];
        let start = scope.namespace_range.0 as usize;
        let end = scope.namespace_range.1 as usize;

        &self.namespace[start..end]
    }

    pub fn find_in_namespace(
        &self,
        scope_id: ScopeId,
        symbol_id: SymbolId,
    ) -> Option<DeclarationId> {
        self.get_namespace_entries(scope_id)
            .iter()
            .find(|(entry_symbol_id, _)| *entry_symbol_id == symbol_id)
            .map(|(_, declaration_id)| *declaration_id)
    }

    pub fn get_nth_namespace_entry(
        &self,
        scope_id: ScopeId,
        index: usize,
    ) -> Option<(SymbolId, DeclarationId)> {
        self.get_namespace_entries(scope_id).get(index).copied()
    }

    pub fn namespace_len(&self, scope_id: ScopeId) -> usize {
        let scope = &self.scopes[scope_id.index()];

        (scope.namespace_range.1 - scope.namespace_range.0) as usize
    }

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
    namespace_range: (u32, u32),
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
        match definition {
            Definition::Module { .. }
            | Definition::Use { .. }
            | Definition::Function { .. }
            | Definition::NativeFunction { .. }
            | Definition::StructType { .. }
            | Definition::EnumType { .. }
            | Definition::Variant { .. }
            | Definition::TypeAlias { .. }
            | Definition::Constant { .. }
            | Definition::Trait { .. }
            | Definition::InherentImplementation { .. }
            | Definition::TraitImplementation { .. }
            | Definition::InherentAssociatedConstant { .. }
            | Definition::InherentAssociatedType { .. }
            | Definition::TraitAssociatedConstant { .. }
            | Definition::TraitAssociatedType { .. }
            | Definition::Placeholder => false,

            Definition::Local { .. } | Definition::Field { .. } => matches!(
                self,
                ScopeKind::Item | ScopeKind::Associated | ScopeKind::Constant
            ),

            Definition::TypeParameter => matches!(self, ScopeKind::Item | ScopeKind::Constant),
        }
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

    pub fn inner(self) -> u32 {
        self.0.get()
    }

    fn index(self) -> usize {
        (self.0.get() - 1) as usize
    }
}

pub struct ScopeFrame {
    pub parent_scope_id: ScopeId,
    pub start_index: usize,
}
