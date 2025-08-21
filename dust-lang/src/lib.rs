//! The Dust programming language library.
#![feature(
    allocator_api,
    formatting_options,
    hash_set_entry,
    iter_intersperse,
    new_range_api,
    never_type,
    new_zeroed_alloc,
    offset_of_enum,
    once_cell_get_mut,
    pattern,
    pointer_try_cast_aligned,
    result_option_map_or_default
)]

pub mod chunk;
pub mod compiler;
pub mod dust_error;
pub mod instruction;
// pub mod jit_vm;
pub mod lexer;
pub mod native_function;
pub mod parser;
pub mod program;
pub mod span;
pub mod syntax_tree;
pub mod token;
pub mod r#type;
pub mod value;

// #[cfg(test)]
// mod tests;

pub use chunk::{Chunk, Disassembler, TuiDisassembler};
pub use compiler::{ChunkCompiler, CompileError, compile};
// pub use dust_error::{AnnotatedError, DustError, ErrorMessage};
pub use instruction::{
    Address, Instruction, InstructionFields, MemoryKind, OperandType, Operation,
};
// pub use jit_vm::{
//     Cell,
//     JIT_ERROR_TEXT,
//     JitCompiler,
//     JitError,
//     JitLogic,
//     JitVm,
//     Object,
//     Register,
//     Thread,
//     ThreadResult,
//     // run,
// };
pub use lexer::{LexError, Lexer};
pub use native_function::NativeFunction;
pub use program::Program;
pub use span::Span;
pub use token::Token;
pub use r#type::{FunctionType, Type, TypeConflict};
pub use value::{List, Value};

#[cfg(feature = "global-mimalloc")]
mod allocator {
    use mimalloc::MiMalloc;

    #[global_allocator]
    static GLOBAL: MiMalloc = MiMalloc;
}
