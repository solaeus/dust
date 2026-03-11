//! Type instance collection that stores every type known to the `Compiler`.

use std::{
    cmp::Ordering,
    hash::{Hash, Hasher},
    ops::Range,
};

use indexmap::{IndexSet, set::MutableValues};
use smallvec::SmallVec;

use crate::resolver::{declaration_graph::DeclarationId, error::ResolverError};

/// Type instance collection that stores every type known to the `Compiler`.
#[derive(Debug)]
pub struct TypeGraph {
    types: IndexSet<TypeNode>,
    members: Vec<TypeId>,
    next_inferred_type_id: InferredTypeId,
}

impl TypeGraph {
    pub fn new() -> Self {
        let mut type_graph = Self {
            types: IndexSet::new(),
            members: Vec::new(),
            next_inferred_type_id: InferredTypeId(0),
        };

        let _unit_type_id = type_graph.add_type(TypeNode::unit_type());
        let _boolean_type_id = type_graph.add_type(TypeNode::Boolean);
        let _character_type_id = type_graph.add_type(TypeNode::Character);
        let _u8_type_id = type_graph.add_type(TypeNode::UnsignedInteger(UnsignedIntegerType::U8));
        let _i8_type_id = type_graph.add_type(TypeNode::SignedInteger(SignedIntegerType::I8));
        let _u16_type_id = type_graph.add_type(TypeNode::UnsignedInteger(UnsignedIntegerType::U16));
        let _i16_type_id = type_graph.add_type(TypeNode::SignedInteger(SignedIntegerType::I16));
        let _u32_type_id = type_graph.add_type(TypeNode::UnsignedInteger(UnsignedIntegerType::U32));
        let _i32_type_id = type_graph.add_type(TypeNode::SignedInteger(SignedIntegerType::I32));
        let _u64_type_id = type_graph.add_type(TypeNode::UnsignedInteger(UnsignedIntegerType::U64));
        let _i64_type_id = type_graph.add_type(TypeNode::SignedInteger(SignedIntegerType::I64));
        let _u128_type_id =
            type_graph.add_type(TypeNode::UnsignedInteger(UnsignedIntegerType::U128));
        let _i128_type_id = type_graph.add_type(TypeNode::SignedInteger(SignedIntegerType::I128));
        let _f32_type_id = type_graph.add_type(TypeNode::Float(FloatType::F32));
        let _f64_type_id = type_graph.add_type(TypeNode::Float(FloatType::F64));
        let _never_type_id = type_graph.add_type(TypeNode::Never);

        debug_assert_eq!(_unit_type_id, TypeId::UNIT);
        debug_assert_eq!(_boolean_type_id, TypeId::BOOLEAN);
        debug_assert_eq!(_u8_type_id, TypeId::U_8);
        debug_assert_eq!(_i8_type_id, TypeId::I_8);
        debug_assert_eq!(_u16_type_id, TypeId::U_16);
        debug_assert_eq!(_i16_type_id, TypeId::I_16);
        debug_assert_eq!(_u32_type_id, TypeId::U_32);
        debug_assert_eq!(_i32_type_id, TypeId::I_32);
        debug_assert_eq!(_u64_type_id, TypeId::U_64);
        debug_assert_eq!(_i64_type_id, TypeId::I_64);
        debug_assert_eq!(_u128_type_id, TypeId::U_128);
        debug_assert_eq!(_i128_type_id, TypeId::I_128);
        debug_assert_eq!(_f32_type_id, TypeId::F_32);
        debug_assert_eq!(_f64_type_id, TypeId::F_64);
        debug_assert_eq!(_character_type_id, TypeId::CHARACTER);

        type_graph
    }

    pub fn add_type(&mut self, type_node: TypeNode) -> TypeId {
        if let Some(existing) = self.types.get_index_of(&type_node) {
            return TypeId(existing as u32);
        }

        let type_id = TypeId(self.types.len() as u32);

        self.types.insert(type_node);

        type_id
    }

    pub fn get_type(&self, id: TypeId) -> Result<&TypeNode, ResolverError> {
        self.types
            .get_index(id.0 as usize)
            .ok_or(ResolverError::MissingType(id))
    }

    pub fn get_type_mut(&mut self, id: TypeId) -> Result<&mut TypeNode, ResolverError> {
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

    pub fn create_inferred_type(&mut self) -> TypeId {
        let inferred_type_node = TypeNode::Inferred {
            inferred_id: self.next_inferred_type_id,
            resolved: None,
        };

        self.next_inferred_type_id.0 += 1;

        self.add_type(inferred_type_node)
    }
}

/// Type instance identifier.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TypeId(u32);

impl TypeId {
    pub const UNIT: Self = TypeId(0);
    pub const BOOLEAN: Self = TypeId(1);
    pub const CHARACTER: Self = TypeId(2);
    pub const I_8: Self = TypeId(3);
    pub const I_16: Self = TypeId(4);
    pub const I_32: Self = TypeId(5);
    pub const I_64: Self = TypeId(6);
    pub const I_128: Self = TypeId(7);
    pub const U_8: Self = TypeId(8);
    pub const U_16: Self = TypeId(9);
    pub const U_32: Self = TypeId(10);
    pub const U_64: Self = TypeId(11);
    pub const U_128: Self = TypeId(12);
    pub const F_32: Self = TypeId(13);
    pub const F_64: Self = TypeId(14);
    pub const NEVER: Self = TypeId(15);

    pub fn inner(self) -> u32 {
        self.0
    }

    pub fn is_primitive(self) -> bool {
        matches!(
            self,
            Self::BOOLEAN
                | Self::CHARACTER
                | Self::I_8
                | Self::I_16
                | Self::I_32
                | Self::I_64
                | Self::I_128
                | Self::U_8
                | Self::U_16
                | Self::U_32
                | Self::U_64
                | Self::U_128
                | Self::F_32
                | Self::F_64
                | Self::NEVER
        )
    }
}

/// Type representation covering concrete and non-concrete types.
///
/// # Concrete types
///
/// Concrete types are fully known to the compiler and are therefore scalar.
///
/// - All primitive types except the never type are scalar, and therefore concrete.
/// - Composite types whose type members are all concrete are also concrete. `Option<i32>` is
/// concrete because `i32` is concrete, but `Option<T>` is not concrete because `T` is not concrete.
/// - Generics are never concrete. The presence of a generic type means that the `TypeNode`
/// represents the *definition* of a type rather than a specific *instance*.
/// - Inferred types start as non-concrete but may become concrete through type unification. For
/// example, the type of `x` in `let x = 5;` is initially an inferred type, but it becomes concrete
/// when it is unified with the type of `5`, which is `i32`. It is an error if the compiler cannot
/// resolve the type of all used values.
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
pub enum TypeNode {
    /// `bool`: `true` or `false`
    ///
    /// This is a concrete type.
    Boolean,

    /// `char`: a Unicode scalar value.
    ///
    /// This is a concrete type.
    Character,

    /// `i8`, `i16`, `i32`, `i64` and `i128`
    ///
    /// These are concrete types.
    SignedInteger(SignedIntegerType),

    /// `u8`, `u16`, `u32`, `u64` and `u128`
    ///
    /// These are concrete types.
    UnsignedInteger(UnsignedIntegerType),

    /// `f32` and `f64`
    ///
    /// These are concrete types.
    Float(FloatType),

    /// `!`
    ///
    /// The never type, which represents expressions that never return (i.e. infinite loops, panics
    /// and process termination).
    Never,

    /// `(i32, T)`, `(f64, bool, char)`, etc.
    Tuple { element_type_ids: TypeMembers },

    /// `[T; N]`
    Array {
        element_type_id: TypeId,
        length: usize,
    },

    /// `[T]`
    Slice { element_type_id: TypeId },

    /// `fn foo<T>(x: T) -> T`
    ///
    /// The type of a function declaration. Each function has a unique type.
    FunctionDefinition {
        declaration_id: DeclarationId,
        type_arguments: TypeMembers,
    },

    /// `|x: T| x + 1`
    ///
    /// The type of a closure. Each closure has a unique type.
    Closure {
        declaration_id: DeclarationId,
        type_arguments: TypeMembers,
    },

    /// `fn(T) -> T`
    ///
    /// A common type to which function definitions and closures can be coerced.
    FunctionPointer {
        parameter_types: TypeMembers,
        return_type: TypeId,
    },

    /// `struct Foo<T> { x: T }`, `enum Option<T> { Some(T), None }`, etc.
    ///
    /// A composite or sum type.
    Algebraic {
        declaration_id: DeclarationId,
        type_arguments: TypeMembers,
    },

    /// `T` in `fn foo<T>(x: T) -> T`
    ///
    /// A type parameter, i.e. a type that the user did not fully specify. Generic types and any
    /// types that contain generic types (e.g. `Option<T>`) are non-concrete.
    Generic { declaration_id: DeclarationId },

    /// A type that has not yet been resolved.
    ///
    /// Inferred types are created in two situations:
    ///
    /// - The user does not specify a type for a value, e.g. `let x = 5;`
    /// - When instantiating a non-concrete type (i.e. a type that is generic or contains a
    /// generic), those generics are replaced with inferred types. This allows `Option<T>` to be
    /// instantiated as `Option<i32>` where the user writes `Option::Some(5)`.
    Inferred {
        inferred_id: InferredTypeId,
        resolved: Option<TypeId>,
    },
}

impl TypeNode {
    pub fn unit_type() -> Self {
        TypeNode::Tuple {
            element_type_ids: TypeMembers::default(),
        }
    }
}

/// See the [`TypeNode`] documentation for details on equality, ordering and hashing.
impl PartialEq for TypeNode {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (TypeNode::Boolean, TypeNode::Boolean) => true,
            (TypeNode::Character, TypeNode::Character) => true,
            (TypeNode::SignedInteger(left), TypeNode::SignedInteger(right)) => left == right,
            (TypeNode::UnsignedInteger(left), TypeNode::UnsignedInteger(right)) => left == right,
            (TypeNode::Float(left), TypeNode::Float(right)) => left == right,
            (TypeNode::Never, TypeNode::Never) => true,
            (
                TypeNode::Tuple {
                    element_type_ids: left_element_type_ids,
                },
                TypeNode::Tuple {
                    element_type_ids: right_element_type_ids,
                },
            ) => left_element_type_ids == right_element_type_ids,

            (
                TypeNode::Array {
                    element_type_id: left_element_type_id,
                    length: left_length,
                },
                TypeNode::Array {
                    element_type_id: right_element_type_id,
                    length: right_length,
                },
            ) => left_element_type_id == right_element_type_id && left_length == right_length,
            (
                TypeNode::Slice {
                    element_type_id: left_element_type_id,
                },
                TypeNode::Slice {
                    element_type_id: right_element_type_id,
                },
            ) => left_element_type_id == right_element_type_id,
            (
                TypeNode::FunctionDefinition {
                    declaration_id: left_declaration_id,
                    type_arguments: left_type_arguments,
                },
                TypeNode::FunctionDefinition {
                    declaration_id: right_declaration_id,
                    type_arguments: right_type_arguments,
                },
            )
            | (
                TypeNode::Closure {
                    declaration_id: left_declaration_id,
                    type_arguments: left_type_arguments,
                },
                TypeNode::Closure {
                    declaration_id: right_declaration_id,
                    type_arguments: right_type_arguments,
                },
            )
            | (
                TypeNode::Algebraic {
                    declaration_id: left_declaration_id,
                    type_arguments: left_type_arguments,
                },
                TypeNode::Algebraic {
                    declaration_id: right_declaration_id,
                    type_arguments: right_type_arguments,
                },
            ) => {
                left_declaration_id == right_declaration_id
                    && left_type_arguments == right_type_arguments
            }
            (
                TypeNode::FunctionPointer {
                    parameter_types: left_parameter_types,
                    return_type: left_return_type,
                },
                TypeNode::FunctionPointer {
                    parameter_types: right_parameter_types,
                    return_type: right_return_type,
                },
            ) => {
                left_parameter_types == right_parameter_types
                    && left_return_type == right_return_type
            }
            (
                TypeNode::Generic {
                    declaration_id: left_declaration_id,
                },
                TypeNode::Generic {
                    declaration_id: right_declaration_id,
                },
            ) => left_declaration_id == right_declaration_id,
            (
                TypeNode::Inferred {
                    inferred_id: left_inferred_id,
                    resolved: _,
                },
                TypeNode::Inferred {
                    inferred_id: right_inferred_id,
                    resolved: _,
                },
            ) => left_inferred_id == right_inferred_id,
            _ => false,
        }
    }
}

impl Eq for TypeNode {}

impl PartialOrd for TypeNode {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

/// See the [`TypeNode`] documentation for details on equality, ordering and hashing.
impl Ord for TypeNode {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        match (self, other) {
            (TypeNode::Boolean, TypeNode::Boolean) => Ordering::Equal,
            (TypeNode::Boolean, _) => Ordering::Less,
            (TypeNode::Character, TypeNode::Character) => Ordering::Equal,
            (TypeNode::Character, _) => Ordering::Less,
            (TypeNode::SignedInteger(left), TypeNode::SignedInteger(right)) => left.cmp(right),
            (TypeNode::SignedInteger(_), _) => Ordering::Less,
            (TypeNode::UnsignedInteger(left), TypeNode::UnsignedInteger(right)) => left.cmp(right),
            (TypeNode::UnsignedInteger(_), _) => Ordering::Less,
            (TypeNode::Float(left), TypeNode::Float(right)) => left.cmp(right),
            (TypeNode::Float(_), _) => Ordering::Less,
            (TypeNode::Never, TypeNode::Never) => Ordering::Equal,
            (TypeNode::Never, _) => Ordering::Less,
            (
                TypeNode::Tuple {
                    element_type_ids: left_element_type_ids,
                },
                TypeNode::Tuple {
                    element_type_ids: right_element_type_ids,
                },
            ) => left_element_type_ids.cmp(right_element_type_ids),
            (TypeNode::Tuple { .. }, _) => Ordering::Less,
            (
                TypeNode::Array {
                    element_type_id: left_element_type_id,
                    length: left_length,
                },
                TypeNode::Array {
                    element_type_id: right_element_type_id,
                    length: right_length,
                },
            ) => left_element_type_id
                .cmp(right_element_type_id)
                .then_with(|| left_length.cmp(right_length)),
            (TypeNode::Array { .. }, _) => Ordering::Less,
            (
                TypeNode::Slice {
                    element_type_id: left_element_type_id,
                },
                TypeNode::Slice {
                    element_type_id: right_element_type_id,
                },
            ) => left_element_type_id.cmp(right_element_type_id),
            (TypeNode::Slice { .. }, _) => Ordering::Less,
            (
                TypeNode::FunctionDefinition {
                    declaration_id: left_declaration_id,
                    type_arguments: left_type_arguments,
                },
                TypeNode::FunctionDefinition {
                    declaration_id: right_declaration_id,
                    type_arguments: right_type_arguments,
                },
            )
            | (
                TypeNode::Closure {
                    declaration_id: left_declaration_id,
                    type_arguments: left_type_arguments,
                },
                TypeNode::Closure {
                    declaration_id: right_declaration_id,
                    type_arguments: right_type_arguments,
                },
            )
            | (
                TypeNode::Algebraic {
                    declaration_id: left_declaration_id,
                    type_arguments: left_type_arguments,
                },
                TypeNode::Algebraic {
                    declaration_id: right_declaration_id,
                    type_arguments: right_type_arguments,
                },
            ) => left_declaration_id
                .cmp(right_declaration_id)
                .then_with(|| left_type_arguments.cmp(right_type_arguments)),
            (TypeNode::FunctionDefinition { .. }, _)
            | (TypeNode::Closure { .. }, _)
            | (TypeNode::Algebraic { .. }, _) => Ordering::Less,
            (
                TypeNode::FunctionPointer {
                    parameter_types: left_parameter_types,
                    return_type: left_return_type,
                },
                TypeNode::FunctionPointer {
                    parameter_types: right_parameter_types,
                    return_type: right_return_type,
                },
            ) => left_parameter_types
                .cmp(right_parameter_types)
                .then_with(|| left_return_type.cmp(right_return_type)),
            (TypeNode::FunctionPointer { .. }, _) => Ordering::Less,
            (
                TypeNode::Generic {
                    declaration_id: left_declaration_id,
                },
                TypeNode::Generic {
                    declaration_id: right_declaration_id,
                },
            ) => left_declaration_id.cmp(right_declaration_id),
            (TypeNode::Generic { .. }, _) => Ordering::Less,
            (
                TypeNode::Inferred {
                    inferred_id: a_inferred_id,
                    resolved: _,
                },
                TypeNode::Inferred {
                    inferred_id: b_inferred_id,
                    resolved: _,
                },
            ) => a_inferred_id.cmp(b_inferred_id),
            (TypeNode::Inferred { .. }, _) => Ordering::Less,
        }
    }
}

/// See the [`TypeNode`] documentation for details on equality, ordering and hashing.
impl Hash for TypeNode {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            TypeNode::Boolean => {
                state.write_u8(0);
            }
            TypeNode::Character => {
                state.write_u8(1);
            }
            TypeNode::SignedInteger(SignedIntegerType::I8) => {
                state.write_u8(2);
            }
            TypeNode::SignedInteger(SignedIntegerType::I16) => {
                state.write_u8(3);
            }
            TypeNode::SignedInteger(SignedIntegerType::I32) => {
                state.write_u8(4);
            }
            TypeNode::SignedInteger(SignedIntegerType::I64) => {
                state.write_u8(5);
            }
            TypeNode::SignedInteger(SignedIntegerType::I128) => {
                state.write_u8(6);
            }
            TypeNode::UnsignedInteger(UnsignedIntegerType::U8) => {
                state.write_u8(7);
            }
            TypeNode::UnsignedInteger(UnsignedIntegerType::U16) => {
                state.write_u8(8);
            }
            TypeNode::UnsignedInteger(UnsignedIntegerType::U32) => {
                state.write_u8(9);
            }
            TypeNode::UnsignedInteger(UnsignedIntegerType::U64) => {
                state.write_u8(10);
            }
            TypeNode::UnsignedInteger(UnsignedIntegerType::U128) => {
                state.write_u8(11);
            }
            TypeNode::Float(FloatType::F32) => {
                state.write_u8(12);
            }
            TypeNode::Float(FloatType::F64) => {
                state.write_u8(13);
            }
            TypeNode::Tuple { element_type_ids } => {
                state.write_u8(14);
                element_type_ids.hash(state);
            }
            TypeNode::Array {
                element_type_id,
                length,
            } => {
                state.write_u8(15);
                element_type_id.hash(state);
                length.hash(state);
            }
            TypeNode::Slice { element_type_id } => {
                state.write_u8(16);
                element_type_id.hash(state);
            }
            TypeNode::FunctionDefinition {
                declaration_id,
                type_arguments,
            } => {
                state.write_u8(17);
                declaration_id.hash(state);
                type_arguments.hash(state);
            }
            TypeNode::Closure {
                declaration_id,
                type_arguments,
            } => {
                state.write_u8(18);
                declaration_id.hash(state);
                type_arguments.hash(state);
            }
            TypeNode::FunctionPointer {
                parameter_types,
                return_type,
            } => {
                state.write_u8(19);
                parameter_types.hash(state);
                return_type.hash(state);
            }
            TypeNode::Algebraic {
                declaration_id,
                type_arguments,
            } => {
                state.write_u8(20);
                declaration_id.hash(state);
                type_arguments.hash(state);
            }
            TypeNode::Generic { declaration_id } => {
                state.write_u8(21);
                declaration_id.hash(state);
            }
            TypeNode::Inferred {
                inferred_id,
                resolved: _,
            } => {
                state.write_u8(22);
                inferred_id.hash(state);
            }
            TypeNode::Never => {
                state.write_u8(23);
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
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum UnsignedIntegerType {
    U8,
    U16,
    U32,
    U64,
    U128,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum FloatType {
    F32,
    F64,
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::{
        collections::hash_map::DefaultHasher,
        hash::{Hash, Hasher},
    };

    fn hash_of(node: &TypeNode) -> u64 {
        let mut hasher = DefaultHasher::new();
        node.hash(&mut hasher);
        hasher.finish()
    }

    fn algebraic(decl: u32, start: u32, end: u32) -> TypeNode {
        TypeNode::Algebraic {
            declaration_id: DeclarationId(decl),
            type_arguments: TypeMembers { start, end },
        }
    }

    fn function_def(decl: u32, start: u32, end: u32) -> TypeNode {
        TypeNode::FunctionDefinition {
            declaration_id: DeclarationId(decl),
            type_arguments: TypeMembers { start, end },
        }
    }

    fn closure(decl: u32, start: u32, end: u32) -> TypeNode {
        TypeNode::Closure {
            declaration_id: DeclarationId(decl),
            type_arguments: TypeMembers { start, end },
        }
    }

    fn generic(decl: u32) -> TypeNode {
        TypeNode::Generic {
            declaration_id: DeclarationId(decl),
        }
    }

    fn inferred(id: u32, resolved: Option<TypeId>) -> TypeNode {
        TypeNode::Inferred {
            inferred_id: InferredTypeId(id),
            resolved,
        }
    }

    #[test]
    fn eq_hash_consistency() {
        let cases: Vec<(TypeNode, TypeNode, bool)> = vec![
            (TypeNode::Boolean, TypeNode::Boolean, true),
            (TypeNode::Boolean, TypeNode::Character, false),
            (
                TypeNode::SignedInteger(SignedIntegerType::I32),
                TypeNode::SignedInteger(SignedIntegerType::I32),
                true,
            ),
            (
                TypeNode::SignedInteger(SignedIntegerType::I32),
                TypeNode::SignedInteger(SignedIntegerType::I64),
                false,
            ),
            (
                TypeNode::UnsignedInteger(UnsignedIntegerType::U32),
                TypeNode::SignedInteger(SignedIntegerType::I32),
                false,
            ),
            (
                TypeNode::Float(FloatType::F32),
                TypeNode::Float(FloatType::F64),
                false,
            ),
            (algebraic(1, 0, 1), algebraic(1, 0, 1), true),
            (algebraic(1, 0, 1), algebraic(1, 1, 2), false),
            (algebraic(1, 0, 1), algebraic(2, 0, 1), false),
            (function_def(1, 0, 1), function_def(1, 0, 1), true),
            (function_def(1, 0, 1), function_def(1, 1, 2), false),
            (closure(1, 0, 1), closure(1, 0, 1), true),
            (closure(1, 0, 1), closure(1, 1, 2), false),
            (generic(1), generic(1), true),
            (generic(1), generic(2), false),
            (inferred(0, None), inferred(0, Some(TypeId::BOOLEAN)), true),
            (inferred(0, None), inferred(1, None), false),
            (TypeNode::Never, TypeNode::Never, true),
        ];

        for (i, (a, b, expect_eq)) in cases.iter().enumerate() {
            assert_eq!(a == b, *expect_eq, "case {i}: eq");
            if *expect_eq {
                assert_eq!(hash_of(a), hash_of(b), "case {i}: eq but different hash");
            }
        }
    }

    #[test]
    fn eq_ord_consistency() {
        let cases: Vec<(TypeNode, TypeNode)> = vec![
            (algebraic(1, 0, 1), algebraic(1, 5, 9)),
            (function_def(1, 0, 1), function_def(1, 5, 9)),
            (closure(1, 0, 1), closure(1, 5, 9)),
            (inferred(0, None), inferred(0, Some(TypeId::BOOLEAN))),
        ];

        for (i, (a, b)) in cases.iter().enumerate() {
            assert_eq!(
                a == b,
                a.cmp(b) == Ordering::Equal,
                "case {i}: PartialEq and Ord disagree",
            );
        }
    }

    #[test]
    fn index_set_deduplication() {
        let cases: Vec<(TypeNode, TypeNode, usize)> = vec![
            (algebraic(1, 0, 1), algebraic(1, 0, 1), 1),
            (algebraic(1, 0, 1), algebraic(1, 1, 2), 2),
            (algebraic(1, 0, 1), algebraic(2, 0, 1), 2),
            (function_def(1, 0, 1), function_def(1, 1, 2), 2),
            (TypeNode::Boolean, TypeNode::Boolean, 1),
        ];

        for (i, (a, b, expected_len)) in cases.iter().enumerate() {
            let mut set = IndexSet::new();
            set.insert(*a);
            set.insert(*b);
            assert_eq!(set.len(), *expected_len, "case {i}");
        }
    }
}
