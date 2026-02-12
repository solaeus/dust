//! The Dust programming language library.
#![expect(incomplete_features)]
#![feature(
    formatting_options,
    generic_const_exprs,
    int_from_ascii,
    iterator_try_collect
)]

pub mod compiler;
pub mod disassembler;
pub mod dust_crate;
pub mod dust_error;
pub mod jit_vm;
pub mod lexer;
pub mod parser;
pub mod project;
pub mod prototype;
pub mod source;
pub mod token;
pub mod r#type;
pub mod value;

mod constant_table;
mod instruction;
mod native_function;

#[cfg(test)]
mod tests;

#[cfg(feature = "global-mimalloc")]
mod allocator {
    use mimalloc::MiMalloc;

    #[global_allocator]
    static GLOBAL: MiMalloc = MiMalloc;
}
