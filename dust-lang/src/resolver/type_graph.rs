use std::{
    hash::{Hash, Hasher},
    ops::Range,
};

use indexmap::{IndexSet, set::MutableValues};

use crate::{
    compiler::error::{CompileError, InternalCompileError},
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

        let _none_type_id = type_graph.add_type(TypeNode::None);
        let _boolean_type_id = type_graph.add_type(TypeNode::Boolean);
        let _byte_type_id = type_graph.add_type(TypeNode::Byte);
        let _character_type_id = type_graph.add_type(TypeNode::Character);
        let _float_type_id = type_graph.add_type(TypeNode::Float);
        let _integer_type_id = type_graph.add_type(TypeNode::Integer);
        let _string_type_id = type_graph.add_type(TypeNode::String);

        debug_assert_eq!(_none_type_id, TypeId::NONE);
        debug_assert_eq!(_boolean_type_id, TypeId::BOOLEAN);
        debug_assert_eq!(_byte_type_id, TypeId::BYTE);
        debug_assert_eq!(_character_type_id, TypeId::CHARACTER);
        debug_assert_eq!(_float_type_id, TypeId::FLOAT);
        debug_assert_eq!(_integer_type_id, TypeId::INTEGER);
        debug_assert_eq!(_string_type_id, TypeId::STRING);

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

    pub fn get_type(&self, id: TypeId) -> Result<&TypeNode, CompileError> {
        self.types
            .get_index(id.0 as usize)
            .ok_or(CompileError::Internal(InternalCompileError::MissingType(
                id,
            )))
    }

    pub fn get_type_mut(&mut self, id: TypeId) -> Result<&mut TypeNode, CompileError> {
        self.types
            .get_index_mut2(id.0 as usize)
            .ok_or(CompileError::Internal(InternalCompileError::MissingType(
                id,
            )))
    }

    pub fn add_type_members(&mut self, types: &[TypeId]) -> TypeMembers {
        let members = TypeMembers {
            start: self.members.len() as u32,
            count: types.len() as u32,
        };

        self.members.extend_from_slice(types);

        members
    }

    pub fn get_type_members(&self, members: TypeMembers) -> Result<&[TypeId], CompileError> {
        self.members
            .get(members.as_usize_range())
            .ok_or(CompileError::Internal(
                InternalCompileError::MissingTypeMembers(members),
            ))
    }

    pub fn get_type_member(&self, index: u32) -> Result<&TypeId, CompileError> {
        self.members
            .get(index as usize)
            .ok_or(CompileError::Internal(
                InternalCompileError::MissingTypeMember(index),
            ))
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
    pub const NONE: Self = TypeId(0);
    pub const BOOLEAN: Self = TypeId(1);
    pub const BYTE: Self = TypeId(2);
    pub const CHARACTER: Self = TypeId(3);
    pub const FLOAT: Self = TypeId(4);
    pub const INTEGER: Self = TypeId(5);
    pub const STRING: Self = TypeId(6);

    pub fn inner(self) -> u32 {
        self.0
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum TypeNode {
    None,
    Boolean,
    Byte,
    Character,
    Float,
    Integer,
    String,
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
        generics: DeclarationMembers,
        fields: DeclarationMembers,
    },
    Enum {
        declaration_id: DeclarationId,
        generics: DeclarationMembers,
        variants: DeclarationMembers,
    },
    Inferred {
        inferred_id: InferredTypeId,
        resolved: Option<TypeId>,
    },
}

impl Hash for TypeNode {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            TypeNode::None => state.write_u8(0),
            TypeNode::Boolean => state.write_u8(1),
            TypeNode::Byte => state.write_u8(2),
            TypeNode::Character => state.write_u8(3),
            TypeNode::Float => state.write_u8(4),
            TypeNode::Integer => state.write_u8(5),
            TypeNode::String => state.write_u8(6),
            TypeNode::List { element_type } => {
                state.write_u8(7);
                element_type.hash(state);
            }
            TypeNode::Function {
                type_parameters,
                value_parameters,
                return_type_id,
            } => {
                state.write_u8(8);
                type_parameters.hash(state);
                value_parameters.hash(state);
                return_type_id.hash(state);
            }
            TypeNode::Struct {
                declaration_id,
                generics,
                fields,
            } => {
                state.write_u8(9);
                declaration_id.hash(state);
                generics.hash(state);
                fields.hash(state);
            }
            TypeNode::Enum {
                declaration_id,
                generics,
                variants,
            } => {
                state.write_u8(10);
                declaration_id.hash(state);
                generics.hash(state);
                variants.hash(state);
            }
            TypeNode::Inferred { inferred_id, .. } => {
                state.write_u8(11);
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
