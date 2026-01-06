//! Built-in functions that implement extended functionality.

use std::fmt::{self, Display, Formatter};

use serde::{Deserialize, Serialize};

use crate::resolver::{FunctionTypeNode, Resolver, TypeId, TypeNode};

/// A Dust-native function.
///
/// See the [module-level documentation](index.html) for more information.
#[derive(Clone, Copy, Debug, Eq, PartialEq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct NativeFunction {
    pub index: u16,
}

impl NativeFunction {
    pub fn no_op_signature(resolver: &mut Resolver) -> TypeId {
        resolver.add_type_node(TypeNode::Function(FunctionTypeNode {
            type_parameters: (0, 0),
            value_parameters: (0, 0),
            return_type: TypeId::NONE,
        }))
    }

    pub fn read_line_signature(resolver: &mut Resolver) -> TypeId {
        resolver.add_type_node(TypeNode::Function(FunctionTypeNode {
            type_parameters: (0, 0),
            value_parameters: (0, 0),
            return_type: TypeId::STRING,
        }))
    }

    pub fn write_line_signature(resolver: &mut Resolver) -> TypeId {
        let value_parameters = resolver.add_type_members(&[TypeId::STRING]);

        resolver.add_type_node(TypeNode::Function(FunctionTypeNode {
            type_parameters: (0, 0),
            value_parameters,
            return_type: TypeId::NONE,
        }))
    }

    pub fn spawn_signature(resolver: &mut Resolver) -> TypeId {
        let function_argument_type_id =
            resolver.add_type_node(TypeNode::Function(FunctionTypeNode {
                type_parameters: (0, 0),
                value_parameters: (0, 0),
                return_type: TypeId::NONE,
            }));
        let value_parameters = resolver.add_type_members(&[function_argument_type_id]);

        resolver.add_type_node(TypeNode::Function(FunctionTypeNode {
            type_parameters: (0, 0),
            value_parameters,
            return_type: TypeId::NONE,
        }))
    }
}

macro_rules! define_native_functions {
    (
        $count: literal,
        $(($index: literal, $name: expr, $const_name: ident, $argument_count: literal)),
        *
    ) => {

        impl NativeFunction {
            $(
                pub const $const_name: NativeFunction = NativeFunction { index: $index };
            )*

            pub const ALL: [NativeFunction; $count] = [
                $(
                    NativeFunction { index: $index },
                )*
            ];

            pub fn from_index(index: u16) -> Self {
                NativeFunction {
                    index,
                }
            }

            pub fn name(&self) -> &'static str {
                match self.index {
                    $(
                        $index => $name,
                    )*
                    _ => unreachable!(),
                }
            }

            pub fn argument_count(&self) -> u16 {
                match self.index {
                    $(
                        $index => $argument_count,
                    )*
                    _ => unreachable!(),
                }
            }

            #[allow(clippy::should_implement_trait)]
            pub fn from_str(string: &str) -> Option<Self> {
                match string {
                    $(
                        $name => Some(NativeFunction {
                            index: $index,
                        }),
                    )*
                    _ => None,
                }
            }
        }

        impl Display for NativeFunction {
            fn fmt(&self, f: &mut Formatter) -> fmt::Result {
                match self.index {
                    $(
                        $index => write!(f, "{}", $name),
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
        0,
        "no_op",
        NO_OP,
        0
    ),
    (
        1,
        "read_line",
        READ_LINE,
        0
    ),
    (
        2,
        "write_line",
        WRITE_LINE,
        1
    ),
    (
        4,
        "spawn",
        SPAWN,
        1
    )
}
