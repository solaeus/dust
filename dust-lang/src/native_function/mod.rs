//! Built-in functions that implement extended functionality.

use std::fmt::{self, Display, Formatter};

use serde::{Deserialize, Serialize};

use crate::r#type::{FunctionType, Type};

macro_rules! define_native_function {
    ($(($index: literal, $name:expr, $type:expr)),*) => {
        /// A Dust-native function.
        ///
        /// See the [module-level documentation](index.html) for more information.
        #[derive(Clone, Copy, Debug, Eq, PartialEq, PartialOrd, Ord, Serialize, Deserialize)]
        pub struct NativeFunction {
            pub index: u16,

        }

        impl NativeFunction {
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

define_native_function! {
    (
        0,
        "no_op",
        FunctionType::new([], [], Type::None)
    ),
    (
        1,
        "read_line",
        FunctionType::new([], [], Type::String)
    ),
    (
        2,
        "write_line",
        FunctionType::new([], [Type::String], Type::None)
    ),
    (
        4,
        "spawn",
        FunctionType::new([], [ Type::function([], [], Type::None)], Type::None)
    )
}

impl NativeFunction {
    pub const ALL: [NativeFunction; 4] = [
        NativeFunction::NO_OP,
        NativeFunction::READ_LINE,
        NativeFunction::WRITE_LINE,
        NativeFunction::SPAWN,
    ];
    pub const NO_OP: NativeFunction = NativeFunction { index: 0 };
    pub const READ_LINE: NativeFunction = NativeFunction { index: 1 };
    pub const WRITE_LINE: NativeFunction = NativeFunction { index: 2 };
    pub const SPAWN: NativeFunction = NativeFunction { index: 4 };
}
