//! The Dust programming language library.
#![feature(
    allocator_api,
    formatting_options,
    hash_set_entry,
    int_from_ascii,
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
pub mod constant_table;
pub mod dust_crate;
pub mod dust_error;
pub mod instruction;
pub mod jit_vm;
pub mod lexer;
pub mod native_function;
pub mod parser;
pub mod position;
pub mod project;
pub mod resolver;
pub mod source;
pub mod syntax_tree;
pub mod token;
pub mod r#type;
pub mod value;

#[cfg(test)]
mod tests;

pub use chunk::Chunk;
pub use compiler::{ChunkCompiler, CompileError, compile_main};
pub use constant_table::ConstantTable;
pub use dust_error::{AnnotatedError, DustError};
pub use instruction::{
    Address, Instruction, InstructionFields, MemoryKind, OperandType, Operation,
};
pub use source::Source;
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
pub use lexer::{Lexer, tokenize};
pub use native_function::NativeFunction;
pub use position::{Position, Span};
pub use resolver::Resolver;
pub use token::Token;
pub use r#type::{FunctionType, Type};
pub use value::{List, Value};

#[cfg(feature = "global-mimalloc")]
mod allocator {
    use mimalloc::MiMalloc;

    #[global_allocator]
    static GLOBAL: MiMalloc = MiMalloc;
}
