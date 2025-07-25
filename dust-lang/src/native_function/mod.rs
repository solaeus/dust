//! Built-in functions that implement extended functionality.
//!
//! Native functions are used to implement features that are not possible to implement in Dust
//! itself or that are more efficient to implement in Rust.
mod convert;
mod io;
mod thread;

use std::fmt::{self, Display, Formatter};

use serde::{Deserialize, Serialize};

use tracing::warn;

use crate::{FunctionType, Type, vm::CallFrame};

#[allow(non_camel_case_types)]
pub type NativeFunctionLogic =
    unsafe extern "C" fn(thread: &mut crate::vm::Thread, frame: &mut CallFrame);

macro_rules! define_native_function {
    ($(($index: literal, $name:expr, $type:expr, $logic:expr)),*) => {
        /// A Dust-native function.
        ///
        /// See the [module-level documentation](index.html) for more information.
        #[derive(Clone, Debug, Eq, PartialEq, PartialOrd, Ord, Serialize, Deserialize)]
        pub struct NativeFunction {
            pub index: usize,

        }

        impl NativeFunction {
            pub fn from_index(index: usize) -> Self {
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

#[unsafe(no_mangle)]
pub extern "C" fn no_op(_thread: &mut crate::vm::Thread, _frame: &mut CallFrame) {
    warn!("Running NO_OP native function")
}

define_native_function! {
    (
        0,
        "_no_op",
        FunctionType::new([], [], Type::None),
        no_op
    ),
    (
        1,
        "_int_to_str",
        FunctionType::new([], [Type::Integer], Type::String),
        convert::int_to_str
    ),
    (
        2,
        "_read_line",
        FunctionType::new([], [], Type::String),
        io::read_line
    ),
    (
        3,
        "_write_line",
        FunctionType::new([], [Type::String], Type::None),
        io::write_line
    ),
    (
        4,
        "_spawn",
        FunctionType::new([], [ Type::function([], [], Type::None)], Type::None),
        thread::spawn
    )
}
