//! Built-in functions that implement extended functionality.

use std::fmt::{self, Display, Formatter};

use serde::{Deserialize, Serialize};

use crate::resolver::{
    declaration_graph::DeclarationMembers,
    type_graph::{TypeGraph, TypeId, TypeMembers, TypeNode},
};

#[derive(Clone, Copy, Debug, Eq, PartialEq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct NativeFunction(pub u16);

impl NativeFunction {
    pub const NO_OP: Self = Self(0);

    // `Vec`
    pub const VEC_WITH_CAPACITY: Self = Self(1);
    pub const VEC_LENGTH: Self = Self(2);
    pub const VEC_INSERT: Self = Self(3);
    pub const VEC_REMOVE: Self = Self(4);
    pub const VEC_CLEAR: Self = Self(5);

    // I/O
    pub const READ_LINE: Self = Self(100);
    pub const WRITE_LINE: Self = Self(101);

    // Threads
    pub const SPAWN_THREAD: Self = Self(200);

    pub fn display_name(self) -> &'static str {
        match self {
            Self::VEC_WITH_CAPACITY => "Vec::with_capacity",
            Self::VEC_LENGTH => "Vec::len",
            Self::VEC_INSERT => "Vec::insert",
            Self::VEC_REMOVE => "Vec::remove",
            Self::VEC_CLEAR => "Vec::clear",
            Self::READ_LINE => "io::read_line",
            Self::WRITE_LINE => "io::write_line",
            Self::SPAWN_THREAD => "io::spawn_thread",
            _ => "no_op",
        }
    }

    pub fn signature(self, types: &mut TypeGraph) -> TypeId {
        match self {
            Self::VEC_WITH_CAPACITY => Self::vec_with_capacity_signature(types),
            Self::VEC_LENGTH => todo!(),
            Self::VEC_INSERT => todo!(),
            Self::VEC_REMOVE => todo!(),
            Self::VEC_CLEAR => todo!(),
            Self::READ_LINE => Self::read_line_signature(types),
            Self::WRITE_LINE => Self::write_line_signature(types),
            Self::SPAWN_THREAD => Self::spawn_thread_signature(types),
            _ => Self::no_op_signature(types),
        }
    }

    fn no_op_signature(types: &mut TypeGraph) -> TypeId {
        types.add_type(TypeNode::Function {
            type_parameters: DeclarationMembers::default(),
            value_parameters: TypeMembers::default(),
            return_type_id: TypeId::UNIT,
        })
    }

    fn vec_with_capacity_signature(types: &mut TypeGraph) -> TypeId {
        let value_parameters = types.add_type_members(&[TypeId::U_64]);
        let element_type_id = types.create_inferred_type();
        let return_type_id = types.add_type(TypeNode::Vec { element_type_id });

        types.add_type(TypeNode::Function {
            type_parameters: DeclarationMembers::default(),
            value_parameters,
            return_type_id,
        })
    }

    fn read_line_signature(types: &mut TypeGraph) -> TypeId {
        types.add_type(TypeNode::Function {
            type_parameters: DeclarationMembers::default(),
            value_parameters: TypeMembers::default(),
            return_type_id: TypeId::STRING,
        })
    }

    fn write_line_signature(types: &mut TypeGraph) -> TypeId {
        let value_parameters = types.add_type_members(&[TypeId::STRING]);

        types.add_type(TypeNode::Function {
            type_parameters: DeclarationMembers::default(),
            value_parameters,
            return_type_id: TypeId::UNIT,
        })
    }

    fn spawn_thread_signature(types: &mut TypeGraph) -> TypeId {
        let argument_type_id = types.add_type(TypeNode::Function {
            type_parameters: DeclarationMembers::default(),
            value_parameters: TypeMembers::default(),
            return_type_id: TypeId::UNIT,
        });
        let value_parameters = types.add_type_members(&[argument_type_id]);

        types.add_type(TypeNode::Function {
            type_parameters: DeclarationMembers::default(),
            value_parameters,
            return_type_id: TypeId::UNIT,
        })
    }
}

impl Display for NativeFunction {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.display_name())
    }
}
