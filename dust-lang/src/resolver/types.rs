//! Type instance collection that stores every type known to the `Compiler`.
use std::{
    cmp::Ordering,
    hash::{Hash, Hasher},
    ops::Range,
};

use indexmap::{IndexSet, set::MutableValues};
use smallvec::SmallVec;

use crate::resolver::{declarations::DeclarationId, error::ResolverError};

/// Type instance collection that stores every type known to the `Compiler`.
#[derive(Debug)]
pub struct Types {
    types: IndexSet<Type>,
    members: Vec<TypeId>,
    next_inferred_type_id: InferredTypeId,
}

impl Types {
    pub fn new() -> Self {
        let mut types = Self {
            types: IndexSet::new(),
            members: Vec::new(),
            next_inferred_type_id: InferredTypeId(0),
        };

        let _unit_type_id = types.add_type(Type::unit_type());
        let _boolean_type_id = types.add_type(Type::Boolean);
        let _i8_type_id = types.add_type(Type::SignedInteger(SignedIntegerType::I8));
        let _i16_type_id = types.add_type(Type::SignedInteger(SignedIntegerType::I16));
        let _i32_type_id = types.add_type(Type::SignedInteger(SignedIntegerType::I32));
        let _i64_type_id = types.add_type(Type::SignedInteger(SignedIntegerType::I64));
        let _i128_type_id = types.add_type(Type::SignedInteger(SignedIntegerType::I128));
        let _isize_type_id = types.add_type(Type::SignedInteger(SignedIntegerType::ISize));
        let _u8_type_id = types.add_type(Type::UnsignedInteger(UnsignedIntegerType::U8));
        let _u16_type_id = types.add_type(Type::UnsignedInteger(UnsignedIntegerType::U16));
        let _u32_type_id = types.add_type(Type::UnsignedInteger(UnsignedIntegerType::U32));
        let _u64_type_id = types.add_type(Type::UnsignedInteger(UnsignedIntegerType::U64));
        let _u128_type_id = types.add_type(Type::UnsignedInteger(UnsignedIntegerType::U128));
        let _usize_type_id = types.add_type(Type::UnsignedInteger(UnsignedIntegerType::USize));
        let _f32_type_id = types.add_type(Type::Float(FloatType::F32));
        let _f64_type_id = types.add_type(Type::Float(FloatType::F64));
        let _character_type_id = types.add_type(Type::Character);
        let _never_type_id = types.add_type(Type::Never);

        debug_assert_eq!(_unit_type_id, TypeId::UNIT);
        debug_assert_eq!(_boolean_type_id, TypeId::BOOLEAN);
        debug_assert_eq!(_i8_type_id, TypeId::I_8);
        debug_assert_eq!(_i16_type_id, TypeId::I_16);
        debug_assert_eq!(_i32_type_id, TypeId::I_32);
        debug_assert_eq!(_i64_type_id, TypeId::I_64);
        debug_assert_eq!(_i128_type_id, TypeId::I_128);
        debug_assert_eq!(_isize_type_id, TypeId::I_SIZE);
        debug_assert_eq!(_u8_type_id, TypeId::U_8);
        debug_assert_eq!(_u16_type_id, TypeId::U_16);
        debug_assert_eq!(_u32_type_id, TypeId::U_32);
        debug_assert_eq!(_u64_type_id, TypeId::U_64);
        debug_assert_eq!(_u128_type_id, TypeId::U_128);
        debug_assert_eq!(_usize_type_id, TypeId::U_SIZE);
        debug_assert_eq!(_f32_type_id, TypeId::F_32);
        debug_assert_eq!(_f64_type_id, TypeId::F_64);
        debug_assert_eq!(_character_type_id, TypeId::CHARACTER);
        debug_assert_eq!(_never_type_id, TypeId::NEVER);

        types
    }

    pub fn add_type(&mut self, type_node: Type) -> TypeId {
        if let Some(existing) = self.types.get_index_of(&type_node) {
            return TypeId(existing as u32);
        }

        let type_id = TypeId(self.types.len() as u32);

        self.types.insert(type_node);

        type_id
    }

    pub fn get_type(&self, id: TypeId) -> Result<&Type, ResolverError> {
        self.types
            .get_index(id.0 as usize)
            .ok_or(ResolverError::MissingType(id))
    }

    pub fn get_type_mut(&mut self, id: TypeId) -> Result<&mut Type, ResolverError> {
        self.types
            .get_index_mut2(id.0 as usize)
            .ok_or(ResolverError::MissingType(id))
    }

    pub fn add_type_members(&mut self, types: SmallVec<[TypeId; 4]>) -> TypeMembers {
        let start = self.members.len() as u32;

        self.members.extend(types);

        let end = self.members.len() as u32;

        TypeMembers { start, end }
    }

    pub fn get_type_members(&self, members: TypeMembers) -> Result<&[TypeId], ResolverError> {
        self.members
            .get(members.as_usize_range())
            .ok_or(ResolverError::MissingTypeMembers(members))
    }

    pub fn get_type_member(&self, index: u32) -> Result<&TypeId, ResolverError> {
        self.members
            .get(index as usize)
            .ok_or(ResolverError::MissingTypeMember(index))
    }

    pub fn create_inferred_type(&mut self, constraint: Option<InferredTypeConstraint>) -> TypeId {
        let inferred_type = Type::Inferred {
            inferred_id: self.next_inferred_type_id,
            constraint,
            resolved: None,
        };
        self.next_inferred_type_id.0 += 1;

        self.add_type(inferred_type)
    }
}

/// Type instance identifier.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TypeId(u32);

impl TypeId {
    pub const UNIT: Self = TypeId(0);
    pub const BOOLEAN: Self = TypeId(1);
    pub const I_8: Self = TypeId(2);
    pub const I_16: Self = TypeId(3);
    pub const I_32: Self = TypeId(4);
    pub const I_64: Self = TypeId(5);
    pub const I_128: Self = TypeId(6);
    pub const I_SIZE: Self = TypeId(7);
    pub const U_8: Self = TypeId(8);
    pub const U_16: Self = TypeId(9);
    pub const U_32: Self = TypeId(10);
    pub const U_64: Self = TypeId(11);
    pub const U_128: Self = TypeId(12);
    pub const U_SIZE: Self = TypeId(13);
    pub const F_32: Self = TypeId(14);
    pub const F_64: Self = TypeId(15);
    pub const CHARACTER: Self = TypeId(16);
    pub const NEVER: Self = TypeId(17);

    pub fn inner(self) -> u32 {
        self.0
    }

    pub fn is_primitive(self) -> bool {
        matches!(
            self,
            Self::UNIT
                | Self::BOOLEAN
                | Self::I_8
                | Self::I_16
                | Self::I_32
                | Self::I_64
                | Self::I_128
                | Self::I_SIZE
                | Self::U_8
                | Self::U_16
                | Self::U_32
                | Self::U_64
                | Self::U_128
                | Self::U_SIZE
                | Self::F_32
                | Self::F_64
                | Self::CHARACTER
                | Self::NEVER
        )
    }
}

/// Type instance representation covering all concrete and non-concrete types.
///
/// # Overview
///
/// - `bool`, `char`, all numeric types and the special pointer type are always concrete. The never
///   type is non-concrete because it does not represent any values.
/// - Composite types whose type members are all concrete are also concrete. `Option<i32>` is
///   concrete because `i32` is concrete, but `Option<T>`, which uses a generic type parameter, will
///   not be concrete until `T` is resolved to a concrete type.
/// - Generics are used by [`Definitions`][]s to represent type parameters that are part of a type
///   definition. They are not used on type instances, so they are never inside of other `Type`
///   variants. Generics are non-concrete by definition.
/// - Inferred types start as non-concrete but may become concrete through type unification. For
///   example, the type of `x` in `let x = 5;` is initially an inferred type, but it becomes concrete
///   when it is unified with an explicit type written elsewhere or the default `i32`.
/// - Types based on a type [`Definition`][] have a `declaration_id` field that can be used to find
///   how it was declared, including its full definition. For each type parameter in the definition,
///   there is a corresponding entry in the `type_arguments` field.
///
/// Because the Rust type system is the primary inspiration for this one, the variants of this enum
/// are very similar to the variants of [`rustc_type_ir::ty_kind::TyKind`][1].
///
/// # PartialEq, Eq, PartialOrd, Ord and Hash
///
/// The `Inferred` variant uses only the `inferred_id` field for equality, ordering and hashing
/// because the `resolved` field is mutable and different unresolved types must be considered unique.
///
/// [1]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_type_ir/ty_kind/enum.TyKind.html
#[derive(Clone, Copy, Debug)]
pub enum Type {
    /// `bool`: `true` or `false` values.
    Boolean,

    /// `char`: the Unicode scalar value type.
    Character,

    /// `i8`, `i16`, `i32`, `i64` and `i128`, which represent signed integers.
    SignedInteger(SignedIntegerType),

    /// `u8`, `u16`, `u32`, `u64` and `u128`, which represent unsigned integers.
    UnsignedInteger(UnsignedIntegerType),

    /// `f32` and `f64`, which represent floating point numbers.
    Float(FloatType),

    /// The never type, which represents expressions that never return (i.e. infinite loops, panics
    /// and process termination).
    ///
    /// `!`
    Never,

    /// An anonymous heterogeneous product type.
    ///
    /// `()`, `(i32, T)`, `(f64, bool, char)`, etc.
    Tuple { element_type_ids: TypeMembers },

    /// An anonymous homogeneous product type with a fixed length.
    ///
    /// `[T; N]`
    Array {
        element_type_id: TypeId,
        length: usize,
    },

    /// A view into a contiguous sequence of elements.
    ///
    /// `[T]`
    Slice {
        declaration_id: DeclarationId,
        element_type_id: TypeId,
    },

    /// An instance of a function definition type.
    ///
    /// ```dust
    /// fn foo<T: Add>(x: T) -> T { ... } // A `fn` item creates the *defintion* of the function type.
    ///
    /// foo::<i32>(42);                   // An *instance* of the function type is created by passing
    /// foo::<u8>(42);                    // type arguments to the function definition type.
    /// ```
    FunctionDefinition {
        declaration_id: DeclarationId,
        type_arguments: TypeMembers,
    },

    /// The anonymous type of a closure expression.
    ///
    /// `|x: T| x + 1`
    Closure {
        value_parameters: TypeMembers,
        return_type_id: TypeId,
    },

    /// A common type with which function definition types and closure types are compatible. The
    /// user cannot write function definition or closure types, they are created from `fn` items and
    /// closure expressions. Instead, the user writes function types and the compiler recognizes
    /// matching function definition and closure types as compatible.
    ///
    /// `fn(T) -> T`
    ///
    /// ```dust
    /// struct MyFunction(fn(i32) -> i32); // The user writes a function type
    ///
    /// fn foo(x: i32) -> i32 { x + 1 }
    ///
    /// fn bar() -> i32 {
    ///     let x = MyFunction(|x| x + 1); // The closure type is compatible
    ///     let y = MyFunction(foo);       // The function definition type is compatible
    ///
    ///     x.0(20) + y.0(20)
    /// }
    /// ```
    Function {
        value_parameters: TypeMembers,
        return_type: TypeId,
    },

    /// A named composite or sum type. Their corresponding type [`Definition`][] contains the fields
    /// or variants of the type.
    ///
    /// ```dust
    /// struct Foo { bar: f32 }
    ///
    /// enum Option<T> {
    ///     Some(T),
    ///     None
    /// }
    /// ```
    Algebraic {
        declaration_id: DeclarationId,
        type_arguments: TypeMembers,
    },

    /// A non-concrete type used in type definitions to represent type arguments to be given when
    /// the type is instantiated.
    ///
    /// `T` in `fn foo<T>(x: T) -> T`
    Generic { declaration_id: DeclarationId },

    /// A type that was not specified by the user and may be resolved to a concrete type through
    /// type unification.
    Inferred {
        inferred_id: InferredTypeId,
        constraint: Option<InferredTypeConstraint>,
        resolved: Option<TypeId>,
    },

    /// An internal type used to represent the pointer fields of types like `Vec` and `String`. The
    /// type arguments make each instance unique but the size is always the size of a pointer on the
    /// target platform.
    Pointer {
        declaration_id: DeclarationId,
        type_arguments: TypeMembers,
    },
}

impl Type {
    pub fn unit_type() -> Self {
        Type::Tuple {
            element_type_ids: TypeMembers::default(),
        }
    }
}

/// See the [`Type`][] documentation for details on equality, ordering and hashing.
impl PartialEq for Type {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Type::Boolean, Type::Boolean) => true,
            (Type::Character, Type::Character) => true,
            (Type::SignedInteger(left), Type::SignedInteger(right)) => left == right,
            (Type::UnsignedInteger(left), Type::UnsignedInteger(right)) => left == right,
            (Type::Float(left), Type::Float(right)) => left == right,
            (Type::Never, Type::Never) => true,
            (
                Type::Tuple {
                    element_type_ids: left_element_type_ids,
                },
                Type::Tuple {
                    element_type_ids: right_element_type_ids,
                },
            ) => left_element_type_ids == right_element_type_ids,

            (
                Type::Array {
                    element_type_id: left_element_type_id,
                    length: left_length,
                },
                Type::Array {
                    element_type_id: right_element_type_id,
                    length: right_length,
                },
            ) => left_element_type_id == right_element_type_id && left_length == right_length,
            (
                Type::Slice {
                    element_type_id: left_element_type_id,
                    ..
                },
                Type::Slice {
                    element_type_id: right_element_type_id,
                    ..
                },
            ) => left_element_type_id == right_element_type_id,
            (
                Type::FunctionDefinition {
                    declaration_id: left_declaration_id,
                    type_arguments: left_type_arguments,
                },
                Type::FunctionDefinition {
                    declaration_id: right_declaration_id,
                    type_arguments: right_type_arguments,
                },
            )
            | (
                Type::Algebraic {
                    declaration_id: left_declaration_id,
                    type_arguments: left_type_arguments,
                },
                Type::Algebraic {
                    declaration_id: right_declaration_id,
                    type_arguments: right_type_arguments,
                },
            ) => {
                left_declaration_id == right_declaration_id
                    && left_type_arguments == right_type_arguments
            }
            (
                Type::Closure {
                    value_parameters: left_value_parameters,
                    return_type_id: left_return_type_id,
                },
                Type::Closure {
                    value_parameters: right_value_parameters,
                    return_type_id: right_return_type_id,
                },
            ) => {
                left_value_parameters == right_value_parameters
                    && left_return_type_id == right_return_type_id
            }
            (
                Type::Function {
                    value_parameters: left_parameter_types,
                    return_type: left_return_type,
                },
                Type::Function {
                    value_parameters: right_parameter_types,
                    return_type: right_return_type,
                },
            ) => {
                left_parameter_types == right_parameter_types
                    && left_return_type == right_return_type
            }
            (
                Type::Generic {
                    declaration_id: left_declaration_id,
                },
                Type::Generic {
                    declaration_id: right_declaration_id,
                },
            ) => left_declaration_id == right_declaration_id,
            (
                Type::Inferred {
                    inferred_id: left_inferred_id,
                    constraint: _,
                    resolved: _,
                },
                Type::Inferred {
                    inferred_id: right_inferred_id,
                    constraint: _,
                    resolved: _,
                },
            ) => left_inferred_id == right_inferred_id,
            _ => false,
        }
    }
}

impl Eq for Type {}

impl PartialOrd for Type {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

/// See the [`Type`][] documentation for details on equality, ordering and hashing.
impl Ord for Type {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        match (self, other) {
            (Type::Boolean, Type::Boolean) => Ordering::Equal,
            (Type::Boolean, _) => Ordering::Less,
            (Type::Character, Type::Character) => Ordering::Equal,
            (Type::Character, _) => Ordering::Less,
            (Type::SignedInteger(left), Type::SignedInteger(right)) => left.cmp(right),
            (Type::SignedInteger(_), _) => Ordering::Less,
            (Type::UnsignedInteger(left), Type::UnsignedInteger(right)) => left.cmp(right),
            (Type::UnsignedInteger(_), _) => Ordering::Less,
            (Type::Float(left), Type::Float(right)) => left.cmp(right),
            (Type::Float(_), _) => Ordering::Less,
            (Type::Never, Type::Never) => Ordering::Equal,
            (Type::Never, _) => Ordering::Less,
            (
                Type::Tuple {
                    element_type_ids: left_element_type_ids,
                },
                Type::Tuple {
                    element_type_ids: right_element_type_ids,
                },
            ) => left_element_type_ids.cmp(right_element_type_ids),
            (Type::Tuple { .. }, _) => Ordering::Less,
            (
                Type::Array {
                    element_type_id: left_element_type_id,
                    length: left_length,
                },
                Type::Array {
                    element_type_id: right_element_type_id,
                    length: right_length,
                },
            ) => left_element_type_id
                .cmp(right_element_type_id)
                .then_with(|| left_length.cmp(right_length)),
            (Type::Array { .. }, _) => Ordering::Less,
            (
                Type::Slice {
                    element_type_id: left_element_type_id,
                    ..
                },
                Type::Slice {
                    element_type_id: right_element_type_id,
                    ..
                },
            ) => left_element_type_id.cmp(right_element_type_id),
            (Type::Slice { .. }, _) => Ordering::Less,
            (
                Type::FunctionDefinition {
                    declaration_id: left_declaration_id,
                    type_arguments: left_type_arguments,
                },
                Type::FunctionDefinition {
                    declaration_id: right_declaration_id,
                    type_arguments: right_type_arguments,
                },
            ) => left_declaration_id
                .cmp(right_declaration_id)
                .then_with(|| left_type_arguments.cmp(right_type_arguments)),
            (Type::FunctionDefinition { .. }, _) => Ordering::Less,
            (
                Type::Closure {
                    value_parameters: left_value_parameters,
                    return_type_id: left_return_type_id,
                },
                Type::Closure {
                    value_parameters: right_value_parameters,
                    return_type_id: right_return_type_id,
                },
            ) => left_value_parameters
                .cmp(right_value_parameters)
                .then_with(|| left_return_type_id.cmp(right_return_type_id)),
            (Type::Closure { .. }, _) => Ordering::Less,
            (
                Type::Function {
                    value_parameters: left_parameter_types,
                    return_type: left_return_type,
                },
                Type::Function {
                    value_parameters: right_parameter_types,
                    return_type: right_return_type,
                },
            ) => left_parameter_types
                .cmp(right_parameter_types)
                .then_with(|| left_return_type.cmp(right_return_type)),
            (Type::Function { .. }, _) => Ordering::Less,
            (
                Type::Algebraic {
                    declaration_id: left_declaration_id,
                    type_arguments: left_type_arguments,
                },
                Type::Algebraic {
                    declaration_id: right_declaration_id,
                    type_arguments: right_type_arguments,
                },
            ) => left_declaration_id
                .cmp(right_declaration_id)
                .then_with(|| left_type_arguments.cmp(right_type_arguments)),
            (Type::Algebraic { .. }, _) => Ordering::Less,
            (
                Type::Generic {
                    declaration_id: left_declaration_id,
                },
                Type::Generic {
                    declaration_id: right_declaration_id,
                },
            ) => left_declaration_id.cmp(right_declaration_id),
            (Type::Generic { .. }, _) => Ordering::Less,
            (
                Type::Inferred {
                    inferred_id: a_inferred_id,
                    constraint: _,
                    resolved: _,
                },
                Type::Inferred {
                    inferred_id: b_inferred_id,
                    constraint: _,
                    resolved: _,
                },
            ) => a_inferred_id.cmp(b_inferred_id),
            (Type::Inferred { .. }, _) => Ordering::Less,
            (
                Type::Pointer {
                    declaration_id: left_declaration_id,
                    type_arguments: left_type_arguments,
                },
                Type::Pointer {
                    declaration_id: right_declaration_id,
                    type_arguments: right_type_arguments,
                },
            ) => left_declaration_id
                .cmp(right_declaration_id)
                .then_with(|| left_type_arguments.cmp(right_type_arguments)),
            (Type::Pointer { .. }, _) => Ordering::Less,
        }
    }
}

/// See the [`Type`][] documentation for details on equality, ordering and hashing.
impl Hash for Type {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            Type::Boolean => {
                state.write_u8(0);
            }
            Type::Character => {
                state.write_u8(1);
            }
            Type::SignedInteger(SignedIntegerType::I8) => {
                state.write_u8(2);
            }
            Type::SignedInteger(SignedIntegerType::I16) => {
                state.write_u8(3);
            }
            Type::SignedInteger(SignedIntegerType::I32) => {
                state.write_u8(4);
            }
            Type::SignedInteger(SignedIntegerType::I64) => {
                state.write_u8(5);
            }
            Type::SignedInteger(SignedIntegerType::I128) => {
                state.write_u8(6);
            }
            Type::SignedInteger(SignedIntegerType::ISize) => {
                state.write_u8(7);
            }
            Type::UnsignedInteger(UnsignedIntegerType::U8) => {
                state.write_u8(8);
            }
            Type::UnsignedInteger(UnsignedIntegerType::U16) => {
                state.write_u8(9);
            }
            Type::UnsignedInteger(UnsignedIntegerType::U32) => {
                state.write_u8(10);
            }
            Type::UnsignedInteger(UnsignedIntegerType::U64) => {
                state.write_u8(11);
            }
            Type::UnsignedInteger(UnsignedIntegerType::U128) => {
                state.write_u8(12);
            }
            Type::UnsignedInteger(UnsignedIntegerType::USize) => {
                state.write_u8(13);
            }
            Type::Float(FloatType::F32) => {
                state.write_u8(14);
            }
            Type::Float(FloatType::F64) => {
                state.write_u8(15);
            }
            Type::Never => {
                state.write_u8(16);
            }
            Type::Tuple { element_type_ids } => {
                state.write_u8(17);
                element_type_ids.hash(state);
            }
            Type::Array {
                element_type_id,
                length,
            } => {
                state.write_u8(18);
                element_type_id.hash(state);
                length.hash(state);
            }
            Type::Slice {
                element_type_id, ..
            } => {
                state.write_u8(19);
                element_type_id.hash(state);
            }
            Type::FunctionDefinition {
                declaration_id,
                type_arguments,
            } => {
                state.write_u8(20);
                declaration_id.hash(state);
                type_arguments.hash(state);
            }
            Type::Closure {
                value_parameters,
                return_type_id,
            } => {
                state.write_u8(21);
                value_parameters.hash(state);
                return_type_id.hash(state);
            }
            Type::Function {
                value_parameters,
                return_type,
            } => {
                state.write_u8(22);
                value_parameters.hash(state);
                return_type.hash(state);
            }
            Type::Algebraic {
                declaration_id,
                type_arguments,
            } => {
                state.write_u8(23);
                declaration_id.hash(state);
                type_arguments.hash(state);
            }
            Type::Generic { declaration_id } => {
                state.write_u8(24);
                declaration_id.hash(state);
            }
            Type::Inferred {
                inferred_id,
                constraint: _,
                resolved: _,
            } => {
                state.write_u8(25);
                inferred_id.hash(state);
            }
            Type::Pointer {
                declaration_id,
                type_arguments,
            } => {
                state.write_u8(26);
                declaration_id.hash(state);
                type_arguments.hash(state);
            }
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TypeMembers {
    start: u32,
    end: u32,
}

impl TypeMembers {
    pub fn as_range(&self) -> Range<u32> {
        self.start..self.end
    }

    pub fn as_usize_range(&self) -> Range<usize> {
        self.start as usize..self.end as usize
    }

    pub fn len(&self) -> u32 {
        self.end - self.start
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct InferredTypeId(u32);

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum SignedIntegerType {
    I8,
    I16,
    I32,
    I64,
    I128,
    ISize,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum UnsignedIntegerType {
    U8,
    U16,
    U32,
    U64,
    U128,
    USize,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum FloatType {
    F32,
    F64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum InferredTypeConstraint {
    Float,
    Integer,
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::{
        collections::hash_map::DefaultHasher,
        hash::{Hash, Hasher},
    };

    fn algebraic(decl: u32, start: u32, end: u32) -> Type {
        Type::Algebraic {
            declaration_id: DeclarationId(decl),
            type_arguments: TypeMembers { start, end },
        }
    }

    fn function(decl: u32, start: u32, end: u32) -> Type {
        Type::FunctionDefinition {
            declaration_id: DeclarationId(decl),
            type_arguments: TypeMembers { start, end },
        }
    }

    fn closure(start: u32, end: u32, r#return: u32) -> Type {
        Type::Closure {
            value_parameters: TypeMembers { start, end },
            return_type_id: TypeId(r#return),
        }
    }

    fn generic(decl: u32) -> Type {
        Type::Generic {
            declaration_id: DeclarationId(decl),
        }
    }

    fn inferred(id: u32, resolved: Option<TypeId>) -> Type {
        Type::Inferred {
            inferred_id: InferredTypeId(id),
            constraint: None,
            resolved,
        }
    }

    #[test]
    fn eq_hash_consistency() {
        let cases: Vec<(Type, Type, bool)> = vec![
            (Type::Boolean, Type::Boolean, true),
            (Type::Boolean, Type::Character, false),
            (
                Type::SignedInteger(SignedIntegerType::I32),
                Type::SignedInteger(SignedIntegerType::I32),
                true,
            ),
            (
                Type::SignedInteger(SignedIntegerType::I32),
                Type::SignedInteger(SignedIntegerType::I64),
                false,
            ),
            (
                Type::UnsignedInteger(UnsignedIntegerType::U32),
                Type::SignedInteger(SignedIntegerType::I32),
                false,
            ),
            (
                Type::Float(FloatType::F32),
                Type::Float(FloatType::F64),
                false,
            ),
            (algebraic(1, 0, 1), algebraic(1, 0, 1), true),
            (algebraic(1, 0, 1), algebraic(1, 1, 2), false),
            (algebraic(1, 0, 1), algebraic(2, 0, 1), false),
            (function(1, 0, 1), function(1, 0, 1), true),
            (function(1, 0, 1), function(1, 1, 2), false),
            (closure(1, 0, 1), closure(1, 0, 1), true),
            (closure(1, 0, 1), closure(1, 1, 2), false),
            (generic(1), generic(1), true),
            (generic(1), generic(2), false),
            (inferred(0, None), inferred(0, Some(TypeId::BOOLEAN)), true),
            (inferred(0, None), inferred(1, None), false),
            (Type::Never, Type::Never, true),
        ];

        for (a, b, expect_eq) in cases {
            assert_eq!(a == b, expect_eq);

            if expect_eq {
                let a_hash = {
                    let mut hasher = DefaultHasher::new();

                    a.hash(&mut hasher);
                    hasher.finish()
                };
                let b_hash = {
                    let mut hasher = DefaultHasher::new();

                    b.hash(&mut hasher);
                    hasher.finish()
                };

                assert_eq!(a_hash, b_hash);
            }
        }
    }

    #[test]
    fn eq_ord_consistency() {
        let cases: Vec<(Type, Type)> = vec![
            (algebraic(1, 0, 1), algebraic(1, 5, 9)),
            (function(1, 0, 1), function(1, 5, 9)),
            (closure(1, 0, 1), closure(1, 5, 9)),
            (inferred(0, None), inferred(0, Some(TypeId::BOOLEAN))),
        ];

        for (a, b) in cases {
            assert_eq!(a == b, a.cmp(&b) == Ordering::Equal);
        }
    }
}
