//! The Dust programming language library.
#![expect(incomplete_features)]
#![feature(
    current_thread_id,
    generic_const_exprs,
    iter_array_chunks,
    iterator_try_collect,
    thread_id_value
)]

pub mod compiler;
mod constant_list;
pub mod disassembler;
mod dust_type;
pub mod dust_value;
pub mod error;
mod instruction;
pub mod lexer;
mod native_function;
pub mod parser;
mod program;
pub mod project;
mod prototype;
mod resolver;
pub mod source;
pub mod syntax;
mod token;
pub mod vm;

#[cfg(feature = "mimalloc")]
mod allocator {
    use mimalloc::MiMalloc;

    #[global_allocator]
    static GLOBAL: MiMalloc = MiMalloc;
}
