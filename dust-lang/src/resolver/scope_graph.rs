use smallvec::SmallVec;

use crate::{
    compiler::error::{CompileError, InternalCompileError},
    resolver::declaration_graph::DeclarationId,
};

#[derive(Debug, Default)]
pub struct ScopeGraph {
    scopes: Vec<Scope>,
    members: Vec<ScopeMembers>,
}

impl ScopeGraph {
    pub fn new() -> Self {
        Self {
            scopes: Vec::new(),
            members: Vec::new(),
        }
    }

    pub fn add_scope(&mut self, scope: Scope) -> ScopeId {
        self.add_scope_with_modules_and_imports(scope, SmallVec::new(), SmallVec::new())
    }

    pub fn add_scope_with_modules_and_imports(
        &mut self,
        scope: Scope,
        modules: SmallVec<[ScopeId; 4]>,
        imports: SmallVec<[DeclarationId; 4]>,
    ) -> ScopeId {
        let id = ScopeId(self.scopes.len() as u32);

        self.scopes.push(scope);
        self.members.push(ScopeMembers { modules, imports });

        id
    }

    pub fn get_scope(&self, id: ScopeId) -> Result<&Scope, CompileError> {
        self.scopes.get(id.0 as usize).ok_or(CompileError::Internal(
            InternalCompileError::MissingScope(id),
        ))
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
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ScopeKind {
    Project,
    Module,
    Function,
    Block,
}

#[derive(Debug)]
struct ScopeMembers {
    modules: SmallVec<[ScopeId; 4]>,
    imports: SmallVec<[DeclarationId; 4]>,
}
