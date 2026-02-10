//! The Dust programming language library.
#![expect(incomplete_features)]
#![feature(
    formatting_options,
    generic_const_exprs,
    int_from_ascii,
    iterator_try_collect,
    string_from_utf8_lossy_owned
)]

pub mod compiler;
pub mod disassembler;
pub mod dust_crate;
pub mod dust_error;
pub mod jit_vm;
pub mod lexer;
pub mod parser;
pub mod project;
pub mod source;

mod constant_table;
mod instruction;
mod native_function;
mod prototype;
mod token;
mod r#type;
mod value;

#[cfg(test)]
mod tests;

#[cfg(feature = "global-mimalloc")]
mod allocator {
    use mimalloc::MiMalloc;

    #[global_allocator]
    static GLOBAL: MiMalloc = MiMalloc;
}
