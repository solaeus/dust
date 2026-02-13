use std::ops::Range;

use indexmap::IndexSet;

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
        Self {
            types: IndexSet::new(),
            members: Vec::new(),
            next_inferred_type_id: InferredTypeId(0),
        }
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

    pub fn create_inferred_type(&mut self) -> TypeId {
        let inferred_type_node = TypeNode::Inferred {
            inferred_id: self.next_inferred_type_id,
            resolved: None,
        };

        self.next_inferred_type_id.0 += 1;

        self.add_type(inferred_type_node)
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

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
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
struct InferredTypeId(u32);
