use std::{fmt::Display, path::PathBuf, time::Instant};

use dust_lang::lexer::Lexer;

use crate::{
    cli::{GlobalOptions, InputOptions, OutputOptions, TokenizeCommand},
    handle_source, print_times,
};

pub fn handle_tokenize_command(command: TokenizeCommand, start_time: Instant) {
    let TokenizeCommand {
        global: GlobalOptions { log, time, name },
        input: InputOptions {
            mut eval,
            stdin,
            path,
        },
        output:
            OutputOptions {
                no_output,
                ron,
                pretty_ron,
                postcard,
            },
    } = command;

    fn print(message: impl Display, no_output: bool) {
        if !no_output {
            println!("{message}");
        }
    }

    let source = handle_source(&eval, path, stdin);

    print("# Dust Tokens", no_output);

    for file in source.files() {
        print(format!("\n## {}\n", file.file_name()), no_output);

        let lexer = if file.is_utf8_validated() {
            Lexer::from_utf8(file.content_as_str())
        } else {
            Lexer::from_bytes(file.content_as_bytes())
        };

        for token in lexer {
            print(format!("  - {} at {}", token.kind, token.span), no_output);
        }
    }

    if time {
        let end = start_time.elapsed();

        print_times(&[("Tokenization", end, None)]);
    }
}
