//! The Dust programming language library.
#![expect(incomplete_features)]
#![feature(
    formatting_options,
    generic_const_exprs,
    int_from_ascii,
    iterator_try_collect,
    string_into_chars,
    thread_id_value,
    trim_prefix_suffix,
    uint_bit_width
)]

mod compiler;
mod constant_table;
mod disassembler;
mod dust_error;
mod dust_type;
mod instruction;
mod program;
// mod jit_vm;
mod lexer;
mod native_function;
mod parser;
mod project;
mod prototype;
mod resolver;
mod small_type;
mod source;
mod syntax;
mod token;
mod value;

pub use crate::{
    compiler::{Compiler, compile, compile_main, error::CompileError},
    disassembler::Disassembler,
    dust_error::{Error, ErrorKind},
    dust_type::DustType,
    instruction::Instruction,
    lexer::{Lexer, tokenize_bytes, tokenize_str},
    native_function::NativeFunction,
    parser::{ParseError, ParseResult, Parser, parse},
    program::Program,
    project::{
        DEFAULT_PROGRAM_PATH, EXAMPLE_LIBRARY, EXAMPLE_PROGRAM, PROJECT_CONFIG_PATH, ProgramConfig,
        ProjectConfig,
    },
    prototype::{Prototype, PrototypeId, PrototypeList},
    resolver::Resolver,
    source::{Position, Source, SourceError, SourceFile, SourceFileId, Span},
    syntax::{Syntax, SyntaxKind, SyntaxReader, SyntaxTree, SyntaxVisitor},
    token::{Token, TokenKind},
    value::Value,
};

#[cfg(test)]
mod tests;

#[cfg(feature = "mimalloc")]
mod allocator {
    use mimalloc::MiMalloc;

    #[global_allocator]
    static GLOBAL: MiMalloc = MiMalloc;
}
