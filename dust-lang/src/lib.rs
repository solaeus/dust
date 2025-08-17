//! The Dust programming language library.
#![feature(
    allocator_api,
    formatting_options,
    hash_set_entry,
    new_range_api,
    never_type,
    offset_of_enum,
    once_cell_get_mut,
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

#[cfg(test)]
mod tests;

pub use chunk::{Chunk, Disassembler};
pub use compiler::{Scope, CompileError, Compiler, Global, Item, Local, Path, compile};
pub use dust_error::{AnnotatedError, DustError, ErrorMessage};
pub use instruction::{
    Address, Instruction, InstructionFields, MemoryKind, OperandType, Operation,
};
pub use jit_vm::{
    Cell, JIT_ERROR_TEXT, JitCompiler, JitError, JitLogic, JitVm, Object, Register, Thread,
    ThreadResult, run,
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

#[cfg(feature = "global-mimalloc")]
mod allocator {
    use mimalloc::MiMalloc;

    #[global_allocator]
    static GLOBAL: MiMalloc = MiMalloc;
}

#[derive(
    Clone, Copy, Debug, Default, Eq, PartialEq, PartialOrd, Ord, Hash, Serialize, Deserialize,
)]
pub struct Span(pub usize, pub usize);

impl Display for Span {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({}, {})", self.0, self.1)
    }
}
