use std::{fmt::Display, path::PathBuf, time::Instant};

use dust_lang::lexer::Lexer;

use crate::{handle_source, print_times};

pub fn handle_tokenize_command(
    eval: Option<String>,
    path: Option<PathBuf>,
    stdin: bool,
    no_output: bool,
    time: bool,
    start_time: Instant,
) {
    fn print(message: impl Display, no_output: bool) {
        if !no_output {
            println!("{message}");
        }
    }

    let source = handle_source(&eval, path, stdin);

    print("# Dust Tokens", no_output);

    for file in source.files() {
        print(format!("\n## {}\n", file.file_name().display()), no_output);

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
