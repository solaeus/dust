//! Built-in functions that implement extended functionality.
//!
//! Native functions are used to implement features that are not possible to implement in Dust
//! itself or that are more efficient to implement in Rust.
mod convert;
mod io;
mod thread;

use std::{
    fmt::{self, Display, Formatter},
    marker::PhantomData,
    sync::{Arc, RwLock},
};

use serde::{Deserialize, Serialize};

use tracing::warn;

use crate::{
    Address, Chunk, FunctionType, OperandType, Type,
    vm::{CallFrame, Cell, ThreadPool},
};

pub type NativeFunctionLogic = fn(
    destination: Address,
    arguments: &[(Address, OperandType)],
    call: &mut CallFrame,
    cells: &Arc<RwLock<Vec<Cell>>>,
    threads: &ThreadPool,
);

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
            const LOOKUP_TABLE: [NativeFunctionLogic; 5] = [
                $(
                    $logic,
                )*
            ];

            pub fn from_index(index: usize) -> Self {
                NativeFunction {
                    index,
                }
            }

            pub fn call(
                &self,
                destination: Address,
                arguments: &[(Address, OperandType)],
                call: &mut CallFrame,
                cells: &Arc<RwLock<Vec<Cell>>>,
                threads: &ThreadPool,
            ) {
                Self::LOOKUP_TABLE[self.index as usize](
                    destination,
                    arguments,
                    call,
                    cells,
                    threads,
                );
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

fn no_op(
    _destination: Address,
    _arguments: &[(Address, OperandType)],
    _call: &mut CallFrame,
    _cells: &Arc<RwLock<Vec<Cell>>>,
    _threads: &ThreadPool,
) {
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
