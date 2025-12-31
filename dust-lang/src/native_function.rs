//! Built-in functions that implement extended functionality.

use std::fmt::{self, Display, Formatter};

use serde::{Deserialize, Serialize};

use crate::r#type::{FunctionType, Type};

/// A Dust-native function.
///
/// See the [module-level documentation](index.html) for more information.
#[derive(Clone, Copy, Debug, Eq, PartialEq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct NativeFunction {
    pub index: u16,
}

macro_rules! define_native_functions {
    (
        $count: literal,
        $(($index: literal, $name: expr, $const_name: ident, $type: expr)),
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

            pub fn r#type(&self) -> FunctionType {
                match self.index {
                    $(
                        $index => $type,
                    )*
                    _ => unreachable!(),
                }
            }

            pub fn returns_value(&self) -> bool {
                match self.index {
                    $(
                        $index => $type.return_type != Type::None,
                    )*
                    _ => unreachable!(),
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
        FunctionType::new([], [], Type::None)
    ),
    (
        1,
        "read_line",
        READ_LINE,
        FunctionType::new([], [], Type::String)
    ),
    (
        2,
        "write_line",
        WRITE_LINE,
        FunctionType::new([], [Type::String], Type::None)
    ),
    (
        4,
        "spawn",
        SPAWN,
        FunctionType::new([], [Type::function([], [], Type::None)], Type::None)
    )
}
