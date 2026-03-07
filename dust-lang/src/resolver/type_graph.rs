use std::{
    hash::{Hash, Hasher},
    ops::Range,
};

use indexmap::{IndexSet, set::MutableValues};

use crate::{
    error::{ErrorKind, InternalError},
    resolver::declaration_graph::{DeclarationId, DeclarationMembers},
};

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

        let _unit_type_id = type_graph.add_type(TypeNode::Unit);
        let _boolean_type_id = type_graph.add_type(TypeNode::Boolean);
        let _u8_type_id = type_graph.add_type(TypeNode::U8);
        let _i8_type_id = type_graph.add_type(TypeNode::I8);
        let _u16_type_id = type_graph.add_type(TypeNode::U16);
        let _i16_type_id = type_graph.add_type(TypeNode::I16);
        let _u32_type_id = type_graph.add_type(TypeNode::U32);
        let _i32_type_id = type_graph.add_type(TypeNode::I32);
        let _u64_type_id = type_graph.add_type(TypeNode::U64);
        let _i64_type_id = type_graph.add_type(TypeNode::I64);
        let _u128_type_id = type_graph.add_type(TypeNode::U128);
        let _i128_type_id = type_graph.add_type(TypeNode::I128);
        let _f32_type_id = type_graph.add_type(TypeNode::F32);
        let _f64_type_id = type_graph.add_type(TypeNode::F64);
        let _character_type_id = type_graph.add_type(TypeNode::Character);
        let _string_type_id = type_graph.add_type(TypeNode::String);

        debug_assert_eq!(_unit_type_id, TypeId::UNIT);
        debug_assert_eq!(_boolean_type_id, TypeId::BOOLEAN);
        debug_assert_eq!(_character_type_id, TypeId::CHARACTER);
        debug_assert_eq!(_string_type_id, TypeId::STRING);
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

    pub fn get_type(&self, id: TypeId) -> Result<&TypeNode, ErrorKind> {
        self.types
            .get_index(id.0 as usize)
            .ok_or(ErrorKind::Internal(InternalError::MissingType(id)))
    }

    pub fn get_type_mut(&mut self, id: TypeId) -> Result<&mut TypeNode, ErrorKind> {
        self.types
            .get_index_mut2(id.0 as usize)
            .ok_or(ErrorKind::Internal(InternalError::MissingType(id)))
    }

    pub fn add_type_members(&mut self, types: &[TypeId]) -> TypeMembers {
        let members = TypeMembers {
            start: self.members.len() as u32,
            count: types.len() as u32,
        };

        self.members.extend_from_slice(types);

        members
    }

    pub fn get_type_members(&self, members: TypeMembers) -> Result<&[TypeId], ErrorKind> {
        self.members
            .get(members.as_usize_range())
            .ok_or(ErrorKind::Internal(InternalError::MissingTypeMembers(
                members,
            )))
    }

    pub fn get_type_member(&self, index: u32) -> Result<&TypeId, ErrorKind> {
        self.members
            .get(index as usize)
            .ok_or(ErrorKind::Internal(InternalError::MissingTypeMember(index)))
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

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TypeId(u32);

impl TypeId {
    pub const UNIT: Self = TypeId(0);
    pub const BOOLEAN: Self = TypeId(1);
    pub const U_8: Self = TypeId(2);
    pub const I_8: Self = TypeId(3);
    pub const U_16: Self = TypeId(4);
    pub const I_16: Self = TypeId(5);
    pub const U_32: Self = TypeId(6);
    pub const I_32: Self = TypeId(7);
    pub const U_64: Self = TypeId(8);
    pub const I_64: Self = TypeId(9);
    pub const U_128: Self = TypeId(10);
    pub const I_128: Self = TypeId(11);
    pub const F_32: Self = TypeId(12);
    pub const F_64: Self = TypeId(13);
    pub const CHARACTER: Self = TypeId(14);
    pub const STRING: Self = TypeId(15);

    pub fn inner(self) -> u32 {
        self.0
    }

    pub fn is_primitive(self) -> bool {
        matches!(
            self,
            Self::UNIT
                | Self::BOOLEAN
                | Self::U_8
                | Self::I_8
                | Self::U_16
                | Self::I_16
                | Self::U_32
                | Self::I_32
                | Self::U_64
                | Self::I_64
                | Self::U_128
                | Self::I_128
                | Self::F_32
                | Self::F_64
                | Self::CHARACTER
                | Self::STRING
        )
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum TypeNode {
    Unit,
    Boolean,
    Character,
    String,
    U8,
    I8,
    U16,
    I16,
    U32,
    I32,
    U64,
    I64,
    U128,
    I128,
    F32,
    F64,
    List {
        element_type: TypeId,
    },
    Function {
        type_parameters: DeclarationMembers,
        value_parameters: TypeMembers,
        return_type_id: TypeId,
    },
    Struct {
        declaration_id: DeclarationId,
        type_arguments: TypeMembers,
    },
    Enum {
        declaration_id: DeclarationId,
        type_arguments: TypeMembers,
    },
    Inferred {
        inferred_id: InferredTypeId,
        resolved: Option<TypeId>,
    },
}

impl Hash for TypeNode {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            TypeNode::Unit => state.write_u8(0),
            TypeNode::Boolean => state.write_u8(1),
            TypeNode::Character => state.write_u8(2),
            TypeNode::String => state.write_u8(3),
            TypeNode::U8 => state.write_u8(4),
            TypeNode::I8 => state.write_u8(5),
            TypeNode::U16 => state.write_u8(6),
            TypeNode::I16 => state.write_u8(7),
            TypeNode::U32 => state.write_u8(8),
            TypeNode::I32 => state.write_u8(9),
            TypeNode::U64 => state.write_u8(10),
            TypeNode::I64 => state.write_u8(11),
            TypeNode::U128 => state.write_u8(12),
            TypeNode::I128 => state.write_u8(13),
            TypeNode::F32 => state.write_u8(14),
            TypeNode::F64 => state.write_u8(15),
            TypeNode::List { element_type } => {
                state.write_u8(16);
                element_type.hash(state);
            }
            TypeNode::Function {
                type_parameters,
                value_parameters,
                return_type_id,
            } => {
                state.write_u8(17);
                type_parameters.hash(state);
                value_parameters.hash(state);
                return_type_id.hash(state);
            }
            TypeNode::Struct {
                declaration_id,
                type_arguments,
            } => {
                state.write_u8(18);
                declaration_id.hash(state);
                type_arguments.hash(state);
            }
            TypeNode::Enum {
                declaration_id,
                type_arguments,
            } => {
                state.write_u8(19);
                declaration_id.hash(state);
                type_arguments.hash(state);
            }
            TypeNode::Inferred { inferred_id, .. } => {
                state.write_u8(20);
                inferred_id.hash(state);
            }
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TypeMembers {
    pub start: u32,
    pub count: u32,
}

impl TypeMembers {
    pub fn as_range(&self) -> Range<u32> {
        let start = self.start;
        let end = start.saturating_add(self.count);

        Range { start, end }
    }

    pub fn as_usize_range(&self) -> Range<usize> {
        let start = self.start as usize;
        let end = start.saturating_add(self.count as usize);

        Range { start, end }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct InferredTypeId(u32);
