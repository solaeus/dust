use std::{path::PathBuf, time::Instant};

use dust_lang::{
    dust_error::DustError,
    lexer::Lexer,
    parser::{ParseResult, Parser},
    source::SourceFileId,
};

use crate::{handle_source, print_times};

pub fn handle_parse_command(
    eval: Option<String>,
    path: Option<PathBuf>,
    stdin: bool,
    no_output: bool,
    time: bool,
    start_time: Instant,
) {
    let source = handle_source(eval, path, stdin);
    let mut errors = Vec::new();

    for file in source.files() {
        let lexer = Lexer::new(file.full_source_bytes());
        let parser = Parser::new(SourceFileId(0), lexer);
        let ParseResult {
            syntax_tree,
            errors: parse_errors,
        } = parser.parse_main();

        if !errors.is_empty() {
            errors.extend(parse_errors.into_iter());

            continue;
        }

        if !no_output {
            println!("{syntax_tree}");
        }
    }

    if !errors.is_empty() {
        eprintln!("{}", DustError::parse(errors, source).report());
    }

    if time {
        let parse_time = start_time.elapsed();

        print_times(&[("Parse Time", parse_time, None)]);
    }
}
