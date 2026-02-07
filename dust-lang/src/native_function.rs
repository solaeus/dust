//! Built-in functions that implement extended functionality.

use std::fmt::{self, Display, Formatter};

use serde::{Deserialize, Serialize};

use crate::compiler::{
    Declaration, DeclarationKind, DeclarationMembers, Resolver, ScopeId, TypeId, TypeNode,
};

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
            value_parameters: DeclarationMembers::default(),
            return_type_id: TypeId::NONE,
        })
    }

    pub fn read_line_signature(resolver: &mut Resolver) -> TypeId {
        resolver.add_type(TypeNode::Function {
            type_parameters: DeclarationMembers::default(),
            value_parameters: DeclarationMembers::default(),
            return_type_id: TypeId::STRING,
        })
    }

    pub fn write_line_signature(resolver: &mut Resolver) -> TypeId {
        let argument_symbol = resolver.create_anonymous_symbol();
        let argument_declaration_id = resolver.add_declaration(Declaration {
            symbol: argument_symbol,
            kind: DeclarationKind::Local {
                shadowed: None,
                is_mutable: false,
            },
            scope_id: ScopeId::NONE,
            is_public: true,
            position: None,
        });
        let value_parameters = resolver.add_declaration_members(&[argument_declaration_id]);

        resolver.add_type(TypeNode::Function {
            type_parameters: DeclarationMembers::default(),
            value_parameters,
            return_type_id: TypeId::NONE,
        })
    }

    pub fn spawn_signature(resolver: &mut Resolver) -> TypeId {
        let argument_symbol = resolver.create_anonymous_symbol();
        let argument_declaration_id = resolver.add_declaration(Declaration {
            symbol: argument_symbol,
            kind: DeclarationKind::Local {
                shadowed: None,
                is_mutable: false,
            },
            scope_id: ScopeId::NONE,
            is_public: true,
            position: None,
        });
        let value_parameters = resolver.add_declaration_members(&[argument_declaration_id]);

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
        $(($id: literal, $name: expr, $const_name: ident, $argument_count: literal)),
        *
    ) => {

        impl NativeFunction {
            $(
                pub const $const_name: NativeFunction = NativeFunction { id: $id };
            )*

            pub const ALL: [NativeFunction; $count] = [
                $(
                    NativeFunction { id: $id },
                )*
            ];

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
