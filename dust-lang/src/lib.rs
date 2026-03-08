//! The Dust programming language library.
#![expect(incomplete_features)]
#![feature(
    formatting_options,
    generic_const_exprs,
    int_from_ascii,
    iter_array_chunks,
    iterator_try_collect,
    new_range_api,
    result_option_map_or_default,
    string_into_chars,
    thread_id_value,
    trim_prefix_suffix,
    uint_bit_width
)]

mod compiler;
mod constant_list;
mod disassembler;
mod dust_type;
mod error;
mod instruction;
mod program;
// mod jit_vm;
mod lexer;
mod native_function;
mod parser;
mod project;
mod prototype;
mod resolver;
mod source;
mod syntax;
mod token;
mod value;

pub mod prelude {
    pub use crate::{
        compiler::{Compiler, compile},
        disassembler::Disassembler,
        dust_type::DustType,
        error::{Error, ErrorKind},
        instruction::Instruction,
        lexer::{Lexer, tokenize_bytes, tokenize_str},
        parser::{ParseError, ParseResult, Parser, parse},
        program::Program,
        project::{
            DEFAULT_PROGRAM_PATH, EXAMPLE_LIBRARY, EXAMPLE_PROGRAM, PROJECT_CONFIG_PATH,
            ProgramConfig, ProjectConfig,
        },
        prototype::{Prototype, PrototypeId, PrototypeList},
        resolver::Resolver,
        source::{Position, Source, SourceError, SourceFile, SourceFileId, Span},
        syntax::{Syntax, SyntaxId, SyntaxKind, SyntaxReader, SyntaxTree, SyntaxVisitor},
        token::{Token, TokenKind},
        value::Value,
    };
}

#[cfg(feature = "mimalloc")]
mod allocator {
    use mimalloc::MiMalloc;

    #[global_allocator]
    static GLOBAL: MiMalloc = MiMalloc;
}
