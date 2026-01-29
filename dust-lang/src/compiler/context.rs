use std::{
    collections::HashMap,
    fmt::{self, Display, Formatter},
    hash::{Hash, Hasher},
};

use indexmap::{IndexMap, IndexSet};
use rustc_hash::{FxBuildHasher, FxHasher};
use smallvec::SmallVec;

use crate::{
    compiler::type_graph::{TypeGraph, TypeId},
    constant_table::ConstantTable,
    native_function::NativeFunction,
    prototype::Prototype,
    source::{Position, SourceFileId},
    syntax::SyntaxId,
};

#[derive(Debug)]
pub struct CompileContext {
    pub constants: ConstantTable,

    pub prototypes: Vec<Prototype>,

    pub types: TypeGraph,

    type_bindings: HashMap<SyntaxId, TypeId, FxBuildHasher>,

    declarations: IndexMap<DeclarationKey, Declaration, FxBuildHasher>,

    parameters: IndexSet<DeclarationId, FxBuildHasher>,

    declaration_bindings: HashMap<SyntaxId, DeclarationId, FxBuildHasher>,

    scopes: Vec<Scope>,

    scope_bindings: HashMap<SyntaxId, ScopeId, FxBuildHasher>,
}

impl CompileContext {
    pub fn new() -> Self {
        let mut context = Self {
            constants: ConstantTable::new(),
            prototypes: Vec::new(),
            types: TypeGraph::new(),
            type_bindings: HashMap::default(),
            declarations: IndexMap::default(),
            parameters: IndexSet::default(),
            declaration_bindings: HashMap::default(),
            scopes: vec![],
            scope_bindings: HashMap::default(),
        };

        let _project_scope_id = context.add_scope(Scope {
            kind: ScopeKind::Module,
            parent: ScopeId::PROJECT,
            imports: SmallVec::new(),
            modules: SmallVec::new(),
        });
        let _native_scope_id = context.add_scope(Scope {
            kind: ScopeKind::Module,
            parent: ScopeId::PROJECT,
            imports: SmallVec::new(),
            modules: SmallVec::new(),
        });

        debug_assert_eq!(_project_scope_id, ScopeId::PROJECT);
        debug_assert_eq!(_native_scope_id, ScopeId::NATIVE);

        context.add_native_functions();

        context
    }

    pub fn add_native_functions(&mut self) {
        let no_op_type_id = NativeFunction::no_op_signature(&mut self.types);
        let read_line_type_id = NativeFunction::read_line_signature(&mut self.types);
        let write_line_type_id = NativeFunction::write_line_signature(&mut self.types);
        let spawn_type_id = NativeFunction::spawn_signature(&mut self.types);

        self.add_declaration(
            NativeFunction::NO_OP.name(),
            Declaration {
                kind: DeclarationKind::NativeFunction,
                scope_id: ScopeId::NATIVE,
                type_id: no_op_type_id,
                position: Position::default(),
                is_public: true,
            },
        );
        self.add_declaration(
            NativeFunction::READ_LINE.name(),
            Declaration {
                kind: DeclarationKind::NativeFunction,
                scope_id: ScopeId::NATIVE,
                type_id: read_line_type_id,
                position: Position::default(),
                is_public: true,
            },
        );
        self.add_declaration(
            NativeFunction::WRITE_LINE.name(),
            Declaration {
                kind: DeclarationKind::NativeFunction,
                scope_id: ScopeId::NATIVE,
                type_id: write_line_type_id,
                position: Position::default(),
                is_public: true,
            },
        );
        self.add_declaration(
            NativeFunction::SPAWN.name(),
            Declaration {
                kind: DeclarationKind::NativeFunction,
                scope_id: ScopeId::NATIVE,
                type_id: spawn_type_id,
                position: Position::default(),
                is_public: true,
            },
        );
    }

    pub fn declarations(&self) -> &IndexMap<DeclarationKey, Declaration, FxBuildHasher> {
        &self.declarations
    }

    pub fn add_scope(&mut self, scope: Scope) -> ScopeId {
        let id = ScopeId(self.scopes.len() as u32);

        self.scopes.push(scope);

        id
    }

    pub fn get_scope(&self, id: ScopeId) -> Option<&Scope> {
        self.scopes.get(id.0 as usize)
    }

    pub fn get_scope_mut(&mut self, id: ScopeId) -> Option<&mut Scope> {
        self.scopes.get_mut(id.0 as usize)
    }

    pub fn add_scope_binding(&mut self, syntax_id: SyntaxId, scope_id: ScopeId) {
        self.scope_bindings.insert(syntax_id, scope_id);
    }

    pub fn get_scope_binding(&self, syntax_id: &SyntaxId) -> Option<&ScopeId> {
        self.scope_bindings.get(syntax_id)
    }

    pub fn get_declaration(&self, id: DeclarationId) -> Option<&Declaration> {
        self.declarations
            .get_index(id.0 as usize)
            .map(|(_, declaration)| declaration)
    }

    pub fn add_declaration(&mut self, identifier: &str, declaration: Declaration) -> DeclarationId {
        let symbol = {
            let mut hasher = FxHasher::default();

            identifier.hash(&mut hasher);

            Symbol {
                hash: hasher.finish(),
            }
        };

        let key = DeclarationKey(symbol, declaration.scope_id);

        if let Some((existing_index, _, _)) = self.declarations.get_full(&key) {
            return DeclarationId(existing_index as u32);
        }

        let declaration_id = DeclarationId(self.declarations.len() as u32);

        self.declarations.insert(key, declaration);

        declaration_id
    }

    pub fn get_declaration_mut(
        &mut self,
        declaration_id: &DeclarationId,
    ) -> Option<&mut Declaration> {
        self.declarations
            .get_index_mut(declaration_id.0 as usize)
            .map(|(_, declaration)| declaration)
    }

    pub fn add_declaration_binding(&mut self, syntax_id: SyntaxId, declaration_id: DeclarationId) {
        self.declaration_bindings.insert(syntax_id, declaration_id);
    }

    pub fn get_declaration_binding(&self, syntax_id: &SyntaxId) -> Option<&DeclarationId> {
        self.declaration_bindings.get(syntax_id)
    }

    pub fn add_type_binding(&mut self, syntax_id: SyntaxId, type_id: TypeId) {
        self.type_bindings.insert(syntax_id, type_id);
    }

    pub fn get_type_binding(&self, syntax_id: &SyntaxId) -> Option<&TypeId> {
        self.type_bindings.get(syntax_id)
    }

    pub fn add_parameters(&mut self, parameter_ids: &[DeclarationId]) -> (u32, u32) {
        let start = self.parameters.len() as u32;
        let count = parameter_ids.len() as u32;

        self.parameters.extend(parameter_ids);

        (start, count)
    }

    pub fn get_parameter(&self, index: u32) -> Option<DeclarationId> {
        self.parameters.get_index(index as usize).copied()
    }

    pub fn find_declarations(
        &self,
        identifier: &str,
    ) -> Option<SmallVec<[(DeclarationId, Declaration); 4]>> {
        let symbol = {
            let mut hasher = FxHasher::default();

            identifier.hash(&mut hasher);

            Symbol {
                hash: hasher.finish(),
            }
        };
        let mut found = SmallVec::<[(DeclarationId, Declaration); 4]>::new();

        for (index, (DeclarationKey(found_symbol, _), declaration)) in
            self.declarations.iter().enumerate()
        {
            let declaration_id = DeclarationId(index as u32);

            if *found_symbol == symbol {
                found.push((declaration_id, *declaration));
            }
        }

        if found.is_empty() { None } else { Some(found) }
    }

    pub fn find_declaration_in_scope(
        &self,
        identifier: &str,
        target_scope_id: ScopeId,
    ) -> Option<(DeclarationId, Declaration)> {
        let symbol = {
            let mut hasher = FxHasher::default();

            identifier.hash(&mut hasher);

            Symbol {
                hash: hasher.finish(),
            }
        };

        let mut current_scope_id = target_scope_id;
        let mut current_scope = self.get_scope(current_scope_id)?;

        loop {
            let key = DeclarationKey(symbol, current_scope_id);

            if let Some((index, _, declaration)) = self.declarations.get_full(&key) {
                return Some((DeclarationId(index as u32), *declaration));
            }

            for import_id in &current_scope.imports {
                let import_declaration = self.get_declaration(*import_id)?;
                let key = DeclarationKey(symbol, import_declaration.scope_id);

                if self.declarations.contains_key(&key) {
                    return Some((*import_id, *import_declaration));
                }
            }

            for module_id in &current_scope.modules {
                let module_declaration = self.get_declaration(*module_id)?;
                let key = DeclarationKey(symbol, module_declaration.scope_id);

                if self.declarations.contains_key(&key) {
                    return Some((*module_id, *module_declaration));
                }
            }

            if current_scope.kind != ScopeKind::Block || current_scope_id == ScopeId(0) {
                break;
            }

            current_scope_id = current_scope.parent;
            current_scope = self.get_scope(current_scope_id)?;
        }

        None
    }
}

impl Default for CompileContext {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Symbol {
    hash: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ScopeId(pub u32);

impl ScopeId {
    pub const PROJECT: Self = ScopeId(0);
    pub const NATIVE: Self = ScopeId(1);
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Scope {
    pub kind: ScopeKind,
    pub parent: ScopeId,
    pub imports: SmallVec<[DeclarationId; 4]>,
    pub modules: SmallVec<[DeclarationId; 4]>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ScopeKind {
    Block,
    Function,
    Module,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct DeclarationId(pub u32);

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct DeclarationKey(Symbol, ScopeId);

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Declaration {
    pub kind: DeclarationKind,
    pub scope_id: ScopeId,
    pub type_id: TypeId,
    pub position: Position,
    pub is_public: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum DeclarationKind {
    Function {
        inner_scope_id: ScopeId,
        file_id: SourceFileId,
        syntax_id: SyntaxId,
        parameters: (u32, u32),
        prototype_index: Option<u16>,
    },
    NativeFunction,
    Local {
        shadowed: Option<DeclarationId>,
    },
    LocalMutable {
        shadowed: Option<DeclarationId>,
    },
    Module {
        kind: ModuleKind,
        inner_scope_id: ScopeId,
    },
    Type,
}

impl DeclarationKind {
    pub fn is_local(&self) -> bool {
        matches!(
            self,
            DeclarationKind::Local { .. } | DeclarationKind::LocalMutable { .. }
        )
    }
}

impl Display for DeclarationKind {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            DeclarationKind::Function { .. } => write!(f, "function"),
            DeclarationKind::NativeFunction => write!(f, "native function"),
            DeclarationKind::Local { .. } => write!(f, "local variable"),
            DeclarationKind::LocalMutable { .. } => write!(f, "mutable local variable"),
            DeclarationKind::Module { .. } => write!(f, "module"),
            DeclarationKind::Type => write!(f, "type"),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ModuleKind {
    File,
    Inline,
}
