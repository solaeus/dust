use std::{collections::HashMap, ops::Range};

use rustc_hash::FxBuildHasher;
use smallvec::SmallVec;

use crate::{
    compiler::{
        error::CompileError,
        resolver::{TypeId, scopes::ScopeId, symbols::SymbolId, types::TypeMembers},
    },
    native_function::NativeFunction,
    optimize_inline_capacity,
    source::{Position, SourceCodeId},
    syntax::SyntaxId,
};

#[derive(Debug)]
pub struct Declarations {
    declarations: Vec<Declaration>,
    declaration_lookup: HashMap<(SymbolId, ScopeId), DeclarationId, FxBuildHasher>,
}

impl Declarations {
    pub fn new() -> Self {
        Self {
            declarations: Vec::new(),
            declaration_lookup: HashMap::default(),
        }
    }

    pub fn add_declaration(&mut self, declaration: Declaration) -> DeclarationId {
        let declaration_id = DeclarationId(self.declarations.len() as u32);

        self.declarations.push(declaration);
        self.declaration_lookup.insert(
            (declaration.symbol_id, declaration.scope_id),
            declaration_id,
        );

        declaration_id
    }

    pub fn get_declaration(&self, id: DeclarationId) -> &Declaration {
        &self.declarations[id.0 as usize]
    }

    pub fn reserve_declaration_id(
        &mut self,
        symbol_id: SymbolId,
        scope_id: ScopeId,
        syntax: Option<(Position, SyntaxId)>,
    ) -> DeclarationId {
        let id = DeclarationId(self.declarations.len() as u32);

        self.declarations.push(Declaration {
            symbol_id,
            definition: Definition::Placeholder,
            scope_id,
            syntax,
        });
        self.declaration_lookup.insert((symbol_id, scope_id), id);

        id
    }

    pub fn finish_reserved_range(&mut self) {
        while self.declarations.len() < DeclarationId::RESERVED.end as usize {
            self.declarations.push(Declaration {
                symbol_id: SymbolId::PLACEHOLDER,
                definition: Definition::Placeholder,
                scope_id: ScopeId::CORE,
                syntax: None,
            });
        }
    }

    pub fn set_reserved_declaration(&mut self, id: DeclarationId, definition: Definition) {
        let declaration = &mut self.declarations[id.0 as usize];

        debug_assert_eq!(declaration.definition, Definition::Placeholder);

        declaration.definition = definition;
    }

    pub fn find_declaration_id(
        &self,
        symbol_id: SymbolId,
        scope_id: ScopeId,
    ) -> Option<&DeclarationId> {
        self.declaration_lookup.get(&(symbol_id, scope_id))
    }

    pub fn resolve_forward_reference(
        &mut self,
        forward_id: DeclarationId,
        resolved_id: DeclarationId,
    ) {
        self.declarations[forward_id.0 as usize].definition = Definition::ForwardReference {
            resolved: Some(resolved_id),
        };
    }

    /// Finds the declaration with the given type ID, if it exists. This is O(n) and should only be
    /// used for error reporting or debugging.
    pub fn find_type_declaration(
        &self,
        type_id: TypeId,
    ) -> Result<Option<&Declaration>, CompileError> {
        for declaration in &self.declarations {
            match declaration.definition {
                Definition::Local {
                    type_id: declaration_type_id,
                    ..
                }
                | Definition::Field {
                    type_id: declaration_type_id,
                    ..
                }
                | Definition::Function {
                    return_type_id: declaration_type_id,
                    ..
                }
                | Definition::NativeFunction {
                    return_type_id: declaration_type_id,
                    ..
                }
                | Definition::TypeAlias {
                    aliased_type_id: declaration_type_id,
                    ..
                }
                | Definition::InherentAssociatedType {
                    aliased_type_id: declaration_type_id,
                    ..
                } if declaration_type_id == type_id => {
                    return Ok(Some(declaration));
                }
                _ => {}
            }
        }
        Ok(None)
    }
}

impl Default for Declarations {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct DeclarationId(#[cfg(test)] pub(crate) u32, #[cfg(not(test))] u32);

impl DeclarationId {
    pub type SmallVec = SmallVec<[Self; optimize_inline_capacity::<Self, 4>()]>;

    pub const RESERVED: Range<u32> = 0..100;

    pub const OPTION: Self = Self(0);
    pub const RESULT: Self = Self(1);
    pub const RANGE: Self = Self(2);
    pub const RANGE_INCLUSIVE: Self = Self(3);

    pub fn inner(self) -> u32 {
        self.0
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Declaration {
    pub symbol_id: SymbolId,
    pub scope_id: ScopeId,
    pub definition: Definition,
    pub syntax: Option<(Position, SyntaxId)>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
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
        inner_scope_id: Option<ScopeId>,
    },

    /// A `use` item, which imports an item or enum variant to its scope. When public, it also
    /// exports the item.
    ///
    /// - `use foo::bar;`
    /// - `pub use SomeEnum::Variant;`
    Use {
        public: bool,
        source_declaration_id: DeclarationId,
    },

    /// A `fn` item. This type definition can be instantiated as [`Type::FunctionDefinition`][].
    ///
    /// - `fn yo() { ... }`
    /// - `fn foo<T>(x: T) -> T { ... }`
    Function {
        public: bool,
        parent_impl_or_trait: Option<DeclarationId>,
        type_parameters: Option<ScopeId>,
        value_parameters: Option<ScopeId>,
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
        type_parameters: Option<ScopeId>,
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
        type_parameters: Option<ScopeId>,
        fields: Option<ScopeId>,
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
        type_parameters: Option<ScopeId>,
        variants: Option<ScopeId>,
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
        discriminant: u16,
        enum_declaration_id: DeclarationId,
        fields: Option<ScopeId>,
        kind: VariantKind,
    },

    /// Type parameters have a unique `Type::Generic` type. When a type is instantiated, the type
    /// instance is given a type argument for each type parameter.
    ///
    /// `T` in `fn foo<T>(x: T) -> T { ... }`
    TypeParameter {
        is_self: bool,
    },

    TypeAlias {
        public: bool,
        type_parameters: Option<ScopeId>,
        aliased_type_id: TypeId,
    },

    Constant {
        public: bool,
        type_id: TypeId,
    },

    InherentImplementation {
        type_parameters: Option<ScopeId>,
        self_declaration_id: DeclarationId,
        self_type_arguments: TypeMembers,
        declarations: Option<ScopeId>,
    },

    InherentAssociatedConstant {
        public: bool,
        parent: DeclarationId,
        type_id: TypeId,
    },

    InherentAssociatedType {
        public: bool,
        parent: DeclarationId,
        type_parameters: Option<ScopeId>,
        aliased_type_id: TypeId,
    },

    Trait {
        public: bool,
        type_parameters: Option<ScopeId>,
        supertraits: Option<ScopeId>,
        declarations: Option<ScopeId>,
    },

    TraitImplementation {
        type_parameters: Option<ScopeId>,
        self_declaration_id: DeclarationId,
        self_type_arguments: TypeMembers,
        trait_declaration_id: DeclarationId,
        trait_type_arguments: TypeMembers,
        declarations: Option<ScopeId>,
    },

    TraitAssociatedConstant {
        parent: DeclarationId,
        type_id: TypeId,
        has_default: bool,
    },

    TraitAssociatedType {
        public: bool,
        parent: DeclarationId,
        type_parameters: Option<ScopeId>,
        default_aliased_type_id: Option<TypeId>,
    },

    ForwardReference {
        resolved: Option<DeclarationId>,
    },

    /// Used when reserving a declaration ID.
    Placeholder,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ModuleKind {
    File { source_id: SourceCodeId },
    Inline,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum VariantKind {
    Unit,
    TupleFields,
    NamedFields,
}
