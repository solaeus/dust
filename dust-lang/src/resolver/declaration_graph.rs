use std::{collections::HashMap, ops::Range};

use rustc_hash::FxBuildHasher;
use smallvec::SmallVec;

use crate::{
    native_function::NativeFunction,
    resolver::{
        TypeId, error::ResolverError, scope_graph::ScopeId, symbol_table::SymbolId,
        type_graph::TypeMembers,
    },
    source::{Position, SourceFileId},
    syntax::SyntaxId,
};

#[derive(Debug)]
pub struct DeclarationGraph {
    declarations: Vec<Declaration>,
    declaration_lookup: HashMap<DeclarationKey, DeclarationId, FxBuildHasher>,
    declaration_members: Vec<DeclarationId>,
    declaration_types: HashMap<DeclarationId, TypeId, FxBuildHasher>,
}

impl DeclarationGraph {
    pub fn new() -> Self {
        Self {
            declarations: Vec::new(),
            declaration_lookup: HashMap::default(),
            declaration_members: Vec::new(),
            declaration_types: HashMap::default(),
        }
    }

    pub fn add_declaration(&mut self, declaration: Declaration) -> DeclarationId {
        let key = DeclarationKey {
            symbol_id: declaration.symbol_id,
            scope_id: declaration.scope_id,
        };

        if !matches!(declaration.definition, Definition::Local { .. })
            && let Some(existing_id) = self.declaration_lookup.get(&key)
        {
            return *existing_id;
        }

        let declaration_id = DeclarationId(self.declarations.len() as u32);

        self.declarations.push(declaration);
        self.declaration_lookup.insert(key, declaration_id);

        declaration_id
    }

    pub fn get_declaration(&self, id: DeclarationId) -> Result<&Declaration, ResolverError> {
        self.declarations
            .get(id.0 as usize)
            .ok_or(ResolverError::MissingDeclaration(id))
    }

    pub fn set_declaration_type(&mut self, id: DeclarationId, type_id: TypeId) {
        self.declaration_types.insert(id, type_id);
    }

    pub fn get_declaration_type(&self, id: DeclarationId) -> Result<TypeId, ResolverError> {
        self.declaration_types
            .get(&id)
            .copied()
            .ok_or(ResolverError::MissingDeclarationType(id))
    }

    pub fn find_declaration(
        &self,
        symbol_id: SymbolId,
        scope_id: ScopeId,
        visibility: Visibility,
    ) -> Option<(DeclarationId, &Declaration)> {
        let key = DeclarationKey {
            symbol_id,
            scope_id,
        };

        self.declaration_lookup.get(&key).and_then(|&id| {
            let delcaration = &self.declarations[id.0 as usize];

            match (visibility, delcaration.definition.visibility()) {
                (Visibility::Block, _) => {}
                (Visibility::Module, Visibility::Module) => {}
                _ => return None,
            }

            Some((id, delcaration))
        })
    }

    /// Finds the declaration with the given type ID, if it exists. This is O(n) and should only be
    /// used for error reporting or debugging.
    pub fn find_type_declaration(
        &self,
        type_id: TypeId,
    ) -> Result<Option<&Declaration>, ResolverError> {
        for (declaration_id, declaration_type_id) in &self.declaration_types {
            if *declaration_type_id == type_id {
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
        parameter_ids: SmallVec<[DeclarationId; 4]>,
    ) -> DeclarationMembers {
        let start = self.declaration_members.len() as u32;

        self.declaration_members.extend(parameter_ids);

        let end = self.declaration_members.len() as u32;

        DeclarationMembers { start, end }
    }

    pub fn get_declaration_member(&self, index: u32) -> Result<&DeclarationId, ResolverError> {
        self.declaration_members
            .get(index as usize)
            .ok_or(ResolverError::MissingDeclarationMember(index))
    }

    pub fn get_declaration_members(
        &self,
        members: &DeclarationMembers,
    ) -> Result<&[DeclarationId], ResolverError> {
        self.declaration_members
            .get(members.as_usize_range())
            .ok_or(ResolverError::MissingDeclarationMembers(*members))
    }

    pub fn iter(&self) -> impl Iterator<Item = (DeclarationId, &Declaration)> + '_ {
        self.declarations
            .iter()
            .enumerate()
            .map(|(index, declaration)| (DeclarationId(index as u32), declaration))
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct DeclarationId(#[cfg(test)] pub(crate) u32, #[cfg(not(test))] u32);

impl DeclarationId {
    pub fn inner(self) -> u32 {
        self.0
    }

    pub(crate) fn offset(self, offset: u32) -> Self {
        DeclarationId(self.0 + offset)
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Declaration {
    pub symbol_id: SymbolId,
    pub definition: Definition,
    pub scope_id: ScopeId,
    pub syntax: Option<(Position, SyntaxId)>,
}

#[derive(Clone, Copy, Debug)]
pub enum Definition {
    /// A block-scoped variable created by a `let` statement or a function parameter. Must have an
    /// associated type ID.
    ///
    /// - `let mut x: f64 = 42;`
    /// - `a: f64` in `fn foo(a: f64) { ... }`
    Local {
        mutable: bool,
        shadowed: Option<DeclarationId>,
    },

    /// A namespace that can contain other declarations, either inline or in another file.
    ///
    /// - `mod foo { mod bar { ... } }`
    /// - `mod foo;`
    Module {
        public: bool,
        kind: ModuleKind,
        inner_scope_id: ScopeId,
    },

    /// Definition of a declared function that stores its type and metadata. This type definition
    /// can be instantiated as a [`Type::FunctionDefinition`][].
    ///
    /// - `fn yo() { ... }`
    /// - `fn foo<T>(x: T) -> T { ... }`
    Function {
        public: bool,
        type_parameters: DeclarationMembers,
        value_parameters: TypeMembers,
        return_type_id: TypeId,
    },

    /// Definition of a function declared within the resolver (not by the user) that stores its
    /// type, allowing it to be used like any other function declaration.
    ///
    /// Native functions include:
    ///
    /// - `core::io::print_line`
    /// - `core::vec::Vec::with_capacity`
    /// - `core::string::String::join`
    NativeFunction {
        function: NativeFunction,
        type_parameters: DeclarationMembers,
        value_parameters: TypeMembers,
        return_type_id: TypeId,
    },

    /// Definition of a declared product type. This type definition can be instantiated as
    /// `Type::Algebraic`.
    ///
    /// - `struct Foo<T>(T);`
    /// - `struct Foo { x: f32 }`
    StructType {
        public: bool,
        type_parameters: DeclarationMembers,
        fields: DeclarationMembers,
    },

    /// Definition of a declared sum type. This type definition can be instantiated as
    /// `Type::Algebraic`.
    ///
    /// ```
    /// enum Foo<T> {
    ///     Bar(T),
    ///     Baz { x: f32 }
    ///     Qux,
    /// }
    /// ```
    EnumType {
        public: bool,
        type_parameters: DeclarationMembers,
        variants: DeclarationMembers,
    },

    /// Type parameters have a unique `Type::Generic` type. When a type is instantiated, the type
    /// instance is given a type argument for each type parameter. Must have an associated type ID.
    ///
    /// `T` in `fn foo<T>(x: T) -> T { ... }`
    TypeParameter,

    /// Fields are the members of a product (`struct`) type. Must have an associated type ID.
    ///
    /// `foo: f32` in `struct Bar { foo: f32 }`
    Field { public: bool },
}

impl Definition {
    fn visibility(&self) -> Visibility {
        match self {
            Definition::Local { .. } => Visibility::Block,
            Definition::Module { .. }
            | Definition::Function { .. }
            | Definition::NativeFunction { .. }
            | Definition::StructType { .. }
            | Definition::EnumType { .. }
            | Definition::TypeParameter { .. } => Visibility::Module,
            Definition::Field { .. } => Visibility::Type,
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
    start: u32,
    end: u32,
}

impl DeclarationMembers {
    pub fn len(&self) -> u32 {
        self.end - self.start
    }

    pub fn as_range(&self) -> Range<u32> {
        self.start..self.end
    }

    pub fn as_usize_range(&self) -> Range<usize> {
        self.start as usize..self.end as usize
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ModuleKind {
    File { file_id: SourceFileId },
    Inline,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct DeclarationKey {
    symbol_id: SymbolId,
    scope_id: ScopeId,
}
