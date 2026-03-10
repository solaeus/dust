//! The Dust programming language library.
#![expect(incomplete_features)]
#![feature(
    generic_const_exprs,
    int_from_ascii,
    iter_array_chunks,
    iterator_try_collect
)]

pub mod compiler;
mod constant_list;
pub mod disassembler;
mod dust_type;
pub mod error;
mod instruction;
mod program;
// mod jit_vm;
pub mod lexer;
mod native_function;
pub mod parser;
pub mod project;
mod prototype;
mod resolver;
pub mod source;
pub mod syntax;
mod token;
mod value;

#[cfg(feature = "mimalloc")]
mod allocator {
    use mimalloc::MiMalloc;

    #[global_allocator]
    static GLOBAL: MiMalloc = MiMalloc;
}
