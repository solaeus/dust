use smallvec::SmallVec;

use crate::{
    compiler::{
        error::CompileError,
        resolver::{TypeId, scopes::ScopeId, symbols::SymbolId, types::TypeMembers},
    },
    native_function::NativeFunction,
    optimal_small_vec_inline_capacity,
    source::{FileId, Position, Span},
    syntax::SyntaxId,
};

#[derive(Debug)]
pub struct Declarations {
    declarations: Vec<Declaration>,
}

impl Declarations {
    pub fn new() -> Self {
        Self {
            declarations: Vec::new(),
        }
    }

    pub fn add_declaration(&mut self, declaration: Declaration) -> DeclarationId {
        let declaration_id = DeclarationId(self.declarations.len() as u32);

        self.declarations.push(declaration);

        declaration_id
    }

    pub fn get_declaration(&self, id: DeclarationId) -> Result<&Declaration, CompileError> {
        self.declarations
            .get(id.0 as usize)
            .ok_or(CompileError::MissingDeclaration(id))
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

        id
    }

    pub fn set_reserved_declaration(&mut self, id: DeclarationId, definition: Definition) {
        let declaration = &mut self.declarations[id.0 as usize];

        debug_assert_eq!(declaration.definition, Definition::Placeholder);

        declaration.definition = definition;
    }

    pub fn find_declaration(
        &self,
        symbol_id: SymbolId,
        scope_id: ScopeId,
    ) -> Option<(DeclarationId, &Declaration)> {
        for (index, declaration) in self.declarations.iter().enumerate().rev() {
            if declaration.symbol_id == symbol_id && declaration.scope_id == scope_id {
                return Some((DeclarationId(index as u32), declaration));
            }
        }

        None
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

    pub fn next_declaration_id(&self) -> DeclarationId {
        DeclarationId(self.declarations.len() as u32)
    }

    pub fn iter(&self) -> impl Iterator<Item = (DeclarationId, &Declaration)> + '_ {
        self.declarations
            .iter()
            .enumerate()
            .map(|(index, declaration)| (DeclarationId(index as u32), declaration))
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
    pub type SmallVec = SmallVec<[Self; optimal_small_vec_inline_capacity::<Self>()]>;

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
        inner_scope_id: ScopeId,
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
        type_parameters: ScopeId,
        value_parameters: ScopeId,
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
        type_parameters: ScopeId,
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
        type_parameters: ScopeId,
        fields: ScopeId,
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
        type_parameters: ScopeId,
        variants: ScopeId,
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
        fields: ScopeId,
    },

    /// Type parameters have a unique `Type::Generic` type. When a type is instantiated, the type
    /// instance is given a type argument for each type parameter.
    ///
    /// `T` in `fn foo<T>(x: T) -> T { ... }`
    TypeParameter,

    TypeAlias {
        public: bool,
        type_parameters: ScopeId,
        aliased_type_id: TypeId,
    },

    Constant {
        public: bool,
        type_id: TypeId,
    },

    InherentImplementation {
        type_parameters: ScopeId,
        declarations: ScopeId,
    },

    InherentAssociatedConstant {
        public: bool,
        parent: DeclarationId,
        type_id: TypeId,
    },

    InherentAssociatedType {
        public: bool,
        parent: DeclarationId,
        type_parameters: ScopeId,
        aliased_type_id: TypeId,
    },

    Trait {
        public: bool,
        type_parameters: ScopeId,
        supertraits: ScopeId,
        declarations: ScopeId,
    },

    TraitImplementation {
        type_parameters: ScopeId,
        trait_declaration_id: DeclarationId,
        trait_type_arguments: TypeMembers,
        declarations: ScopeId,
    },

    TraitAssociatedConstant {
        parent: DeclarationId,
        type_id: TypeId,
        has_default: bool,
    },

    TraitAssociatedType {
        parent: DeclarationId,
        type_parameters: ScopeId,
        default_aliased_type_id: Option<TypeId>,
    },

    /// Used when reserving a declaration ID.
    Placeholder,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ModuleKind {
    File { file_id: FileId },
    Inline,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
enum DeclarationDebugInfo {
    Embedded {},
    Source {
        file_id: FileId,
        span: Span,
        syntax_id: SyntaxId,
    },
}
