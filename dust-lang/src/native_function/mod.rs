//! Built-in functions that implement extended functionality.

use std::fmt::{self, Display, Formatter};

use serde::{Deserialize, Serialize};

use crate::{FunctionType, Type};

macro_rules! define_native_function {
    ($(($index: literal, $name:expr, $type:expr)),*) => {
        /// A Dust-native function.
        ///
        /// See the [module-level documentation](index.html) for more information.
        #[derive(Clone, Debug, Eq, PartialEq, PartialOrd, Ord, Serialize, Deserialize)]
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
        "_no_op",
        FunctionType::new([], [], Type::None)
    ),
    (
        1,
        "_int_to_str",
        FunctionType::new([], [Type::Integer], Type::String)
    ),
    (
        2,
        "_read_line",
        FunctionType::new([], [], Type::String)
    ),
    (
        3,
        "_write_line",
        FunctionType::new([], [Type::String], Type::None)
    ),
    (
        4,
        "_spawn",
        FunctionType::new([], [ Type::function([], [], Type::None)], Type::None)
    )
}
