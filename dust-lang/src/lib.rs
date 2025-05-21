//! The Dust programming language library.
//!
//! # Running Programs
//!
//! Dust is easy to embed in another application. The `run` function can be used to run Dust code
//! and get the program's result in a single call.
//!
//! If a program returns a value, it does so with a ConcreteValue. Dust's concrete values are simple
//! and flexible, they are wrappers for familiar Rust types like String, i64 and Vec.
//!
//! If an error occurs, it is returned as a DustError. This error can be used to create a report
//! with source annotations, which should be printed to the user.
//!
//! # Examples
//!
//! ```rust
//! # use dust_lang::{run, Value};
//! let result = run("21 * 2").unwrap();
//!
//! assert_eq!(result, Some(Value::integer(42)));
//! ```
//!
//! ```rust
//! # use dust_lang::{run, DustError};
//! let error = run("21 + wut").unwrap_err();
//! let report = error.report();
//!
//! println!("{}", report);
//! ```
#![feature(new_range_api)]

pub mod chunk;
pub mod compiler;
pub mod dust_error;
pub mod instruction;
pub mod lexer;
pub mod native_function;
pub mod panic;
pub mod panic_vm;
pub mod token;
pub mod r#type;
pub mod value;

#[cfg(test)]
mod tests;

pub use crate::chunk::{Arguments, Chunk, Disassembler, Local, Scope};
pub use crate::compiler::{CompileError, Compiler, compile};
pub use crate::dust_error::{AnnotatedError, DustError};
pub use crate::instruction::{Address, Destination, Instruction, Operation};
pub use crate::lexer::{LexError, Lexer, lex};
pub use crate::native_function::NativeFunction;
pub use crate::panic_vm::{Vm, run};
pub use crate::token::{Token, TokenKind, TokenOwned};
pub use crate::r#type::{EnumType, FunctionType, StructType, Type, TypeConflict};
pub use crate::value::{AbstractList, ConcreteList, ConcreteValue, DustString, Value};

use std::fmt::Display;

use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Eq, PartialEq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Span(pub usize, pub usize);

impl Display for Span {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({}, {})", self.0, self.1)
    }
}
