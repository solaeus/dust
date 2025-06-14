//! Built-in functions that implement extended functionality.
//!
//! Native functions are used to implement features that are not possible to implement in Dust
//! itself or that are more efficient to implement in Rust.
mod io;
mod string;
mod thread;

use std::fmt::{self, Display, Formatter};

use serde::{Deserialize, Serialize};

use tracing::{error, warn};

use crate::{
    Address, FunctionType,
    panic_vm::{CallFrame, Memory, ThreadPool},
    r#type::{Type, TypeKind},
};

const LOOKUP_TABLE: [NativeFunctionLogic; 5] = [
    no_op,
    string::to_string,
    io::read_line,
    io::write_line,
    thread::spawn,
];

pub type NativeFunctionLogic = fn(
    destination: Address,
    arguments: &[(Address, TypeKind)],
    call: &mut CallFrame,
    memory: &mut Memory,
    threads: &ThreadPool,
);

macro_rules! define_native_function {
    ($(($index: literal, $name:ident, $str:expr, $type:expr, $logic:expr)),*) => {

        /// A dust-native function.
        ///
        /// See the [module-level documentation](index.html) for more information.
        #[derive(Clone, Debug, Eq, PartialEq, PartialOrd, Ord, Serialize, Deserialize)]
        pub enum NativeFunction {
            $(
                $name,
            )*
        }

        impl NativeFunction {
            const LOOKUP_TABLE: [NativeFunctionLogic; 5] = [
                $(
                    $logic,
                )*
            ];

            pub fn from_index(index: u16) -> Self {
                match index as usize {
                    $(
                        $index => NativeFunction::$name,
                    )*
                    _ => {
                        error!("Unknown native function index: {index}");

                        NativeFunction::NoOp
                    }
                }
            }

            pub fn call(
                &self,
                destination: Address,
                arguments: &[(Address, TypeKind)],
                call: &mut CallFrame,
                memory: &mut Memory,
                threads: &ThreadPool,
            ) {
                $(
                    LOOKUP_TABLE[$index](
                        destination,
                        arguments,
                        call,
                        memory,
                        threads,
                    );
                )*
            }

            pub fn as_str(&self) -> &'static str {
                match *self {
                    $(
                        NativeFunction::$name => $str,
                    )*
                    _ => unreachable!(),
                }
            }

            #[allow(clippy::should_implement_trait)]
            pub fn from_str(string: &str) -> Option<Self> {
                match string {
                    $(
                        $str => Some(NativeFunction::$name),
                    )*
                    _ => None,
                }
            }

            pub fn r#type(&self) -> FunctionType {
                match *self {
                    $(
                        NativeFunction::$name => $type,
                    )*
                    _ => unreachable!(),
                }
            }

            pub fn returns_value(&self) -> bool {
                match *self {
                    $(
                        NativeFunction::$name => $type.return_type != Type::None,
                    )*
                    _ => unreachable!(),
                }
            }
        }

        impl Display for NativeFunction {
            fn fmt(&self, f: &mut Formatter) -> fmt::Result {
                write!(f, "{}", self.as_str())
            }
        }
    }
}

fn no_op(
    _destination: Address,
    _arguments: &[(Address, TypeKind)],
    _call: &mut CallFrame,
    _memory: &mut Memory,
    _threads: &ThreadPool,
) {
    warn!("Running NO_OP native function")
}

define_native_function! {
    (
        0,
        NoOp,
        "_no_op",
        FunctionType::new([], [], Type::None),
        no_op
    ),
    (
        1,
        ToString,
        "_to_string",
        FunctionType::new([], [Type::Any], Type::String),
        string::to_string
    ),
    (
        2,
        ReadLine,
        "_read_line",
        FunctionType::new([], [], Type::String),
        io::read_line
    ),
    (
        3,
        WriteLine,
        "_write_line",
        FunctionType::new([], [Type::String], Type::None),
        io::write_line
    ),
    (
        4,
        Spawn,
        "_spawn",
        FunctionType::new([], [ Type::function([], [], Type::None)], Type::None),
        thread::spawn
    )
}
