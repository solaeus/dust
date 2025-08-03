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
#![feature(
    formatting_options,
    hash_set_entry,
    new_range_api,
    never_type,
    offset_of_enum,
    once_cell_get_mut,
    panic_payload_as_str,
    pattern,
    new_zeroed_alloc
)]

pub mod chunk;
pub mod compiler;
pub mod dust_crate;
pub mod dust_error;
pub mod instruction;
pub mod jit_vm;
pub mod lexer;
pub mod module;
pub mod native_function;
pub mod program;
pub mod token;
pub mod r#type;
pub mod value;

// #[cfg(test)]
// mod tests;

pub use chunk::{Chunk, Disassembler};
pub use compiler::{BlockScope, CompileError, Compiler, Global, Item, Local, Path, compile};
pub use dust_error::{AnnotatedError, DustError};
pub use instruction::{
    Address, Instruction, InstructionFields, MemoryKind, OperandType, Operation,
};
pub use jit_vm::{
    Cell, JIT_ERROR_TEXT, Jit, JitError, JitExecutor, JitVm, Object, Register, Thread,
    ThreadStatus, run,
};
pub use lexer::{LexError, Lexer, lex};
pub use module::Module;
pub use native_function::NativeFunction;
pub use program::Program;
pub use token::{Token, TokenKind, TokenOwned};
pub use r#type::{FunctionType, Type, TypeConflict};
pub use value::{List, Value};

use std::fmt::Display;

use serde::{Deserialize, Serialize};

#[derive(
    Clone, Copy, Debug, Default, Eq, PartialEq, PartialOrd, Ord, Hash, Serialize, Deserialize,
)]
pub struct Span(pub usize, pub usize);

impl Display for Span {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({}, {})", self.0, self.1)
    }
}
