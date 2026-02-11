//! Built-in functions that implement extended functionality.

use std::fmt::{self, Display, Formatter};

use serde::{Deserialize, Serialize};

use crate::compiler::{DeclarationMembers, Resolver, TypeId, TypeMembers, TypeNode};

/// A Dust-native function.
///
/// See the [module-level documentation](index.html) for more information.
#[derive(Clone, Copy, Debug, Eq, PartialEq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct NativeFunction {
    pub id: u16,
}

impl NativeFunction {
    pub fn no_op_signature(resolver: &mut Resolver) -> TypeId {
        resolver.add_type(TypeNode::Function {
            type_parameters: DeclarationMembers::default(),
            value_parameters: TypeMembers::default(),
            return_type_id: TypeId::NONE,
        })
    }

    pub fn read_line_signature(resolver: &mut Resolver) -> TypeId {
        resolver.add_type(TypeNode::Function {
            type_parameters: DeclarationMembers::default(),
            value_parameters: TypeMembers::default(),
            return_type_id: TypeId::STRING,
        })
    }

    pub fn write_line_signature(resolver: &mut Resolver) -> TypeId {
        let value_parameters = resolver.add_type_members(&[TypeId::STRING]);

        resolver.add_type(TypeNode::Function {
            type_parameters: DeclarationMembers::default(),
            value_parameters,
            return_type_id: TypeId::NONE,
        })
    }

    pub fn spawn_signature(resolver: &mut Resolver) -> TypeId {
        let argument_type_id = resolver.add_type(TypeNode::Function {
            type_parameters: DeclarationMembers::default(),
            value_parameters: TypeMembers::default(),
            return_type_id: TypeId::NONE,
        });
        let value_parameters = resolver.add_type_members(&[argument_type_id]);

        resolver.add_type(TypeNode::Function {
            type_parameters: DeclarationMembers::default(),
            value_parameters,
            return_type_id: TypeId::NONE,
        })
    }
}

macro_rules! define_native_functions {
    (
        $count: literal,
        $(($id: literal, $name: expr, $const_name: ident, $argument_count: literal, $signature: ident)),
        *
    ) => {

        impl NativeFunction {
            pub const COUNT: usize = $count;

            $(
                pub const $const_name: NativeFunction = NativeFunction { id: $id };
            )*

            pub const ALL: [NativeFunction; $count] = [
                $(
                    NativeFunction { id: $id },
                )*
            ];

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

            pub fn argument_count(&self) -> u16 {
                match self.id {
                    $(
                        $id => $argument_count,
                    )*
                    _ => unreachable!(),
                }
            }

            pub fn signature(&self, resolver: &mut Resolver) -> TypeId {
                match self.id {
                    $(
                        $id => Self::$signature(resolver),
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
        0,
        "no_op",
        NO_OP,
        0,
        no_op_signature
    ),
    (
        1,
        "read_line",
        READ_LINE,
        0,
        read_line_signature
    ),
    (
        2,
        "write_line",
        WRITE_LINE,
        1,
        write_line_signature
    ),
    (
        4,
        "spawn",
        SPAWN,
        1,
        spawn_signature
    )
}
