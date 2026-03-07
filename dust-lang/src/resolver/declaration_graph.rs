use std::{collections::HashMap, ops::Range};

use rustc_hash::FxBuildHasher;

use crate::{
    dust_error::{ErrorKind, InternalError},
    native_function::NativeFunction,
    prototype::PrototypeId,
    resolver::{TypeId, scope_graph::ScopeId, symbol_table::SymbolId},
    source::{Position, SourceFileId},
    syntax::SyntaxId,
};

#[derive(Debug)]
pub struct DeclarationGraph {
    declarations: Vec<Declaration>,
    declaration_lookup: HashMap<DeclarationKey, DeclarationId, FxBuildHasher>,
    declaration_members: Vec<DeclarationId>,
    declaration_types: HashMap<DeclarationId, TypeId, FxBuildHasher>,
    declaration_prototypes: HashMap<DeclarationId, PrototypeId, FxBuildHasher>,
}

impl DeclarationGraph {
    pub fn new() -> Self {
        Self {
            declarations: Vec::new(),
            declaration_lookup: HashMap::default(),
            declaration_members: Vec::new(),
            declaration_types: HashMap::default(),
            declaration_prototypes: HashMap::default(),
        }
    }

    pub fn add_declaration(&mut self, declaration: Declaration) -> DeclarationId {
        let key = DeclarationKey {
            symbol: declaration.symbol_id,
            scope_id: declaration.scope_id,
            visibility: declaration.kind.visibility(),
        };

        if declaration.kind != DeclarationKind::Local
            && let Some(existing_id) = self.declaration_lookup.get(&key)
        {
            return *existing_id;
        }

        let declaration_id = DeclarationId(self.declarations.len() as u32);

        self.declarations.push(declaration);
        self.declaration_lookup.insert(key, declaration_id);

        declaration_id
    }

    pub fn get_declaration(&self, id: DeclarationId) -> Result<&Declaration, ErrorKind> {
        self.declarations
            .get(id.0 as usize)
            .ok_or(ErrorKind::Internal(InternalError::MissingDeclaration(id)))
    }

    pub fn find_declaration(
        &self,
        symbol_id: SymbolId,
        scope_id: ScopeId,
        visibility: Visibility,
    ) -> Option<(DeclarationId, &Declaration)> {
        let key = DeclarationKey {
            symbol: symbol_id,
            scope_id,
            visibility,
        };

        self.declaration_lookup.get(&key).map(|&id| {
            let delcaration = &self.declarations[id.0 as usize];

            (id, delcaration)
        })
    }

    /// Finds the declaration with the given type ID, if it exists. This is O(n) and should only be
    /// used for error reporting or debugging.
    pub fn find_type_declaration(
        &self,
        type_id: TypeId,
    ) -> Result<Option<&Declaration>, ErrorKind> {
        for (declaration_id, declaration_type_id) in &self.declaration_types {
            if *declaration_type_id == type_id {
                let declaration = self.get_declaration(*declaration_id)?;

                return Ok(Some(declaration));
            }
        }

        Ok(None)
    }

    /// Finds the declaration with the given prototype ID, if it exists. This is O(n) and should
    /// only be used for error reporting or debugging.
    pub fn find_prototype_declaration(
        &self,
        prototype_id: PrototypeId,
    ) -> Result<Option<&Declaration>, ErrorKind> {
        for (declaration_id, declaration_prototype_id) in &self.declaration_prototypes {
            if *declaration_prototype_id == prototype_id {
                let declaration = self.get_declaration(*declaration_id)?;

                return Ok(Some(declaration));
            }
        }

        Ok(None)
    }

    pub fn next_declaration_id(&self) -> DeclarationId {
        DeclarationId(self.declarations.len() as u32)
    }

    pub fn add_declaration_members(
        &mut self,
        parameter_ids: &[DeclarationId],
    ) -> DeclarationMembers {
        let start = self.declaration_members.len() as u32;
        let count = parameter_ids.len() as u32;

        self.declaration_members.extend(parameter_ids);

        DeclarationMembers { start, count }
    }

    pub fn get_declaration_member(&self, index: u32) -> Result<&DeclarationId, ErrorKind> {
        self.declaration_members
            .get(index as usize)
            .ok_or(ErrorKind::Internal(
                InternalError::MissingDeclarationMember(index),
            ))
    }

    pub fn get_declaration_members(
        &self,
        members: DeclarationMembers,
    ) -> Result<&[DeclarationId], ErrorKind> {
        self.declaration_members
            .get(members.as_usize_range())
            .ok_or(ErrorKind::Internal(
                InternalError::MissingDeclarationMembers(members),
            ))
    }

    pub fn set_declaration_type(&mut self, declaration_id: DeclarationId, type_id: TypeId) {
        self.declaration_types.insert(declaration_id, type_id);
    }

    pub fn get_declaration_type(
        &self,
        declaration_id: &DeclarationId,
    ) -> Result<&TypeId, ErrorKind> {
        self.declaration_types
            .get(declaration_id)
            .ok_or(ErrorKind::Internal(InternalError::MissingDeclarationType(
                *declaration_id,
            )))
    }

    pub fn set_declaration_prototype(
        &mut self,
        declaration_id: DeclarationId,
        prototype_id: PrototypeId,
    ) {
        self.declaration_prototypes
            .insert(declaration_id, prototype_id);
    }

    pub fn get_declaration_prototype(
        &self,
        declaration_id: &DeclarationId,
    ) -> Option<&PrototypeId> {
        self.declaration_prototypes.get(declaration_id)
    }

    pub fn iter(&self) -> impl Iterator<Item = (DeclarationId, &Declaration)> + '_ {
        self.declarations
            .iter()
            .enumerate()
            .map(|(index, declaration)| (DeclarationId(index as u32), declaration))
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct DeclarationId(u32);

impl DeclarationId {
    pub fn inner(self) -> u32 {
        self.0
    }

    pub fn offset(self, offset: u32) -> Self {
        DeclarationId(self.0 + offset)
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Declaration {
    pub symbol_id: SymbolId,
    pub kind: DeclarationKind,
    pub scope_id: ScopeId,
    pub is_public: bool,
    pub syntax: Option<(Position, SyntaxId)>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum DeclarationKind {
    Function,
    NativeFunction(NativeFunction),
    Module {
        kind: ModuleKind,
        inner_scope_id: ScopeId,
    },
    Type {
        parent: Option<DeclarationId>,
        type_parameters: DeclarationMembers,
        members: DeclarationMembers,
    },
    Local,
}

impl DeclarationKind {
    fn visibility(&self) -> Visibility {
        match self {
            DeclarationKind::Function
            | DeclarationKind::NativeFunction(_)
            | DeclarationKind::Module { .. }
            | DeclarationKind::Type { parent: None, .. } => Visibility::Module,
            DeclarationKind::Type {
                parent: Some(_), ..
            } => Visibility::Type,
            DeclarationKind::Local => Visibility::Block,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Visibility {
    Module,
    Block,
    Type,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct DeclarationMembers {
    pub start: u32,
    pub count: u32,
}

impl DeclarationMembers {
    pub fn as_range(&self) -> Range<u32> {
        let start = self.start;
        let end = start.saturating_add(self.count);

        Range { start, end }
    }

    pub fn as_usize_range(&self) -> Range<usize> {
        let start = self.start as usize;
        let end = start.saturating_add(self.count as usize);

        start..end
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ModuleKind {
    File { file_id: SourceFileId },
    Inline,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct DeclarationKey {
    symbol: SymbolId,
    scope_id: ScopeId,
    visibility: Visibility,
}
