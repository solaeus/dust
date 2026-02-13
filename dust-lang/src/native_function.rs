//! Built-in functions that implement extended functionality.

use std::fmt::{self, Display, Formatter};

use serde::{Deserialize, Serialize};

use crate::resolver::{
    declaration_graph::DeclarationMembers,
    type_graph::{TypeGraph, TypeId, TypeMembers, TypeNode},
};

/// A Dust-native function.
///
/// See the [module-level documentation](index.html) for more information.
#[derive(Clone, Copy, Debug, Eq, PartialEq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct NativeFunction {
    pub id: u16,
}

impl NativeFunction {
    pub fn no_op_signature(types: &mut TypeGraph) -> TypeId {
        types.add_type(TypeNode::Function {
            type_parameters: DeclarationMembers::default(),
            value_parameters: TypeMembers::default(),
            return_type_id: TypeId::NONE,
        })
    }

    pub fn read_line_signature(types: &mut TypeGraph) -> TypeId {
        types.add_type(TypeNode::Function {
            type_parameters: DeclarationMembers::default(),
            value_parameters: TypeMembers::default(),
            return_type_id: TypeId::STRING,
        })
    }

    pub fn write_line_signature(types: &mut TypeGraph) -> TypeId {
        let value_parameters = types.add_type_members(&[TypeId::STRING]);

        types.add_type(TypeNode::Function {
            type_parameters: DeclarationMembers::default(),
            value_parameters,
            return_type_id: TypeId::NONE,
        })
    }

    pub fn spawn_thread_signature(types: &mut TypeGraph) -> TypeId {
        let argument_type_id = types.add_type(TypeNode::Function {
            type_parameters: DeclarationMembers::default(),
            value_parameters: TypeMembers::default(),
            return_type_id: TypeId::NONE,
        });
        let value_parameters = types.add_type_members(&[argument_type_id]);

        types.add_type(TypeNode::Function {
            type_parameters: DeclarationMembers::default(),
            value_parameters,
            return_type_id: TypeId::NONE,
        })
    }
}

macro_rules! define_native_functions {
    (
        $count: literal,
        $((
            id: $id: literal,
            name: $name: expr,
            identifier: $const_name: ident,
            signature: $signature: ident
            argument_count: $argument_count: literal,
        )),*
    ) => {
        impl NativeFunction {
            pub const ALL: [NativeFunction; $count] = [
                $(
                    NativeFunction { id: $id },
                )*
            ];

            $(
                pub const $const_name: NativeFunction = NativeFunction { id: $id };
            )*

            #[allow(clippy::should_implement_trait)]
            pub fn from_str(string: &str) -> Option<Self> {
                match string {
                    $(
                        $name => Some(NativeFunction {
                            id: $id,
                        }),
                    )*
                    _ => None,
                }
            }

            pub fn name(&self) -> &'static str {
                match self.id {
                    $(
                        $id => $name,
                    )*
                    _ => unreachable!(),
                }
            }

            pub fn signature(&self, types: &mut TypeGraph) -> TypeId {
                match self.id {
                    $(
                        $id => Self::$signature(types),
                    )*
                    _ => unreachable!(),
                }
            }

            pub fn argument_count(&self) -> u16 {
                match self.id {
                    $(
                        $id => $argument_count,
                    )*
                    _ => unreachable!(),
                }
            }
        }

        impl Display for NativeFunction {
            fn fmt(&self, f: &mut Formatter) -> fmt::Result {
                match self.id {
                    $(
                        $id => write!(f, "{}", $name),
                    )*
                    _ => unreachable!(),
                }
            }
        }
    }
}

define_native_functions! {
    4,
    (
        id: 0,
        name: "no_op",
        identifier: NO_OP,
        signature: no_op_signature
        argument_count: 0,
    ),
    (
        id: 1,
        name: "read_line",
        identifier: READ_LINE,
        signature: read_line_signature
        argument_count: 0,
    ),
    (
        id: 2,
        name: "write_line",
        identifier: WRITE_LINE,
        signature: write_line_signature
        argument_count: 1,
    ),
    (
        id: 4,
        name: "spawn_thread",
        identifier: SPAWN_THREAD,
        signature: spawn_thread_signature
        argument_count: 1,
    )
}
