use std::{collections::HashMap, ops::Range};

use rustc_hash::FxBuildHasher;

use crate::{
    native_function::NativeFunction,
    resolver::{
        TypeId, error::ResolverError, scopes::ScopeId, symbols::SymbolId, types::TypeMembers,
    },
    source::{Position, SourceFileId},
    syntax::SyntaxId,
};

#[derive(Debug)]
pub struct Declarations {
    declarations: Vec<Declaration>,
    declaration_lookup: HashMap<DeclarationKey, DeclarationId, FxBuildHasher>,
    declaration_members: Vec<DeclarationId>,
}

impl Declarations {
    pub fn new() -> Self {
        Self {
            declarations: Vec::new(),
            declaration_lookup: HashMap::default(),
            declaration_members: Vec::new(),
        }
    }

    pub fn add_declaration(&mut self, declaration: Declaration) -> DeclarationId {
        let key = DeclarationKey {
            symbol_id: declaration.symbol_id,
            scope_id: declaration.scope_id,
        };
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

    pub fn reserve_declaration_id(&mut self) -> DeclarationId {
        let id = DeclarationId(self.declarations.len() as u32);

        self.declarations.push(Declaration {
            symbol_id: SymbolId::PLACEHOLDER,
            definition: Definition::Placeholder,
            scope_id: ScopeId::NONE,
            syntax: None,
        });

        id
    }

    pub fn set_declaration(
        &mut self,
        id: DeclarationId,
        declaration: Declaration,
    ) -> Result<(), ResolverError> {
        let index = id.0 as usize;

        if self.declarations.get(index).is_none() {
            return Err(ResolverError::MissingDeclaration(id));
        }

        let key = DeclarationKey {
            symbol_id: declaration.symbol_id,
            scope_id: declaration.scope_id,
        };

        self.declarations[index] = declaration;
        self.declaration_lookup.insert(key, id);

        Ok(())
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
        _type_id: TypeId,
    ) -> Result<Option<&Declaration>, ResolverError> {
        todo!()
    }

    pub fn next_declaration_id(&self) -> DeclarationId {
        DeclarationId(self.declarations.len() as u32)
    }

    pub fn add_declaration_members(
        &mut self,
        parameter_ids: impl IntoIterator<Item = DeclarationId>,
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
    /// A `let` statement or a function value parameter.
    ///
    /// - `let x = 42;`
    /// - `let mut y: u64 = 666;`
    /// - `a: f64` in `fn foo(a: f64) { ... }`
    Local {
        mutable: bool,
        shadowed: Option<DeclarationId>,
        type_id: TypeId,
    },

    /// A `mod` item, which can contain other declarations, either inline or in another file.
    ///
    /// - `mod foo { mod bar { ... } }`
    /// - `mod foo;`
    Module {
        public: bool,
        kind: ModuleKind,
        inner_scope_id: ScopeId,
    },

    /// A `use` item, which imports an item or enum variant to its scope. When public, it also
    /// exports the item.
    ///
    /// - `use foo::bar;`
    /// - `pub use SomeEnum::Variant;`
    Use {
        public: bool,
        item: DeclarationId,
    },

    /// A `fn` item. This type definition can be instantiated as [`Type::FunctionDefinition`][].
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

    /// Definition of a declared product type. A struct type definition can be instantiated as
    /// `Type::Algebraic`.
    ///
    /// - `struct Foo<T>(T);`
    /// - `struct Foo { x: f32 }`
    StructType {
        public: bool,
        type_parameters: DeclarationMembers,
        fields: DeclarationMembers,
    },

    /// Fields are the members of a struct type.
    ///
    /// `foo: f32` in `struct Bar { foo: f32 }`
    Field {
        public: bool,
        parent_struct: DeclarationId,
        type_id: TypeId,
    },

    /// Definition of a declared sum type. This type definition can be instantiated as
    /// `Type::Algebraic`.
    ///
    /// ```dust
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

    /// Variants are the members of an enum type. This is essentially a struct type with a
    /// discriminant field.
    ///
    /// Bar(T) in `enum Foo<T> { Bar(T), ... }`
    ///
    /// ```dust
    /// enum Foo {
    ///   Bar = 0, // If all variants have no fields, the discriminant can be specified manually.
    ///   Baz = 1,
    ///   Qux = 2,
    /// }
    /// ```
    Variant {
        discriminant: u32,
        parent_enum: DeclarationId,
        type_parameters: DeclarationMembers,
        fields: DeclarationMembers,
    },

    /// Type parameters have a unique `Type::Generic` type. When a type is instantiated, the type
    /// instance is given a type argument for each type parameter.
    ///
    /// `T` in `fn foo<T>(x: T) -> T { ... }`
    TypeParameter,

    TypeAlias {
        public: bool,
        type_parameters: DeclarationMembers,
        aliased_type_id: TypeId,
    },

    Constant {
        public: bool,
        type_id: TypeId,
    },

    Trait {
        public: bool,
        inner_scope_id: ScopeId,
        type_parameters: DeclarationMembers,
        supertraits: DeclarationMembers,
        declarations: DeclarationMembers,
    },

    InherentImplementation {
        type_parameters: DeclarationMembers,
        declarations: DeclarationMembers,
    },

    TraitImplementation {
        type_parameters: DeclarationMembers,
        trait_declaration_id: Option<DeclarationId>,
        trait_type_arguments: TypeMembers,
        declarations: DeclarationMembers,
    },

    AssociatedConstant {
        public: bool,
        parent: DeclarationId,
        type_id: TypeId,
    },

    AssociatedType {
        public: bool,
        parent: DeclarationId,
        type_parameters: DeclarationMembers,
        aliased_type_id: TypeId,
    },

    Placeholder,
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
            | Definition::Use { .. }
            | Definition::TypeAlias { .. }
            | Definition::Constant { .. }
            | Definition::Trait { .. }
            | Definition::InherentImplementation { .. }
            | Definition::TraitImplementation { .. } => Visibility::Module,
            Definition::Field { .. }
            | Definition::Variant { .. }
            | Definition::TypeParameter
            | Definition::AssociatedConstant { .. }
            | Definition::AssociatedType { .. }
            | Definition::Placeholder => Visibility::Type,
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

    pub fn is_empty(&self) -> bool {
        self.len() == 0
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
