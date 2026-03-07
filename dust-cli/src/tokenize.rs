use std::fmt::Display;

use dust_lang::prelude::*;

use crate::{
    cli::{GlobalOptions, InputOptions, OutputOptions, TokenizeCommand},
    handle_source,
};

pub fn handle_tokenize_command(command: TokenizeCommand) {
    let TokenizeCommand {
        global: GlobalOptions { log: _, name: _ },
        input: InputOptions { eval, stdin, path },
        output:
            OutputOptions {
                ron: _,
                pretty_ron: _,
                postcard: _,
            },
    } = command;

    let source = match handle_source(&eval, path, stdin) {
        Ok(source) => source,
        Err(error) => error.print_and_exit(),
    };

    println!("# Dust Tokens");

    for file in source.files() {
        println!("\n## {}\n", file.file_name());

        let lexer = if file.is_utf8_validated() {
            Lexer::from_utf8(file.content_as_str())
        } else {
            Lexer::from_bytes(file.content_as_bytes())
        };

        for token in lexer {
            println!("  - {} at {}", token.kind, token.span);
        }
    }
}
