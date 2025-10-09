use std::time::Instant;

use dust_lang::{
    dust_error::DustError,
    lexer::Lexer,
    parser::{ParseResult, Parser},
    source::{Source, SourceCode, SourceFile, SourceFileId},
};

use crate::print_times;

pub fn handle_parse_command(
    eval: Option<String>,
    no_output: bool,
    time: bool,
    start_time: Instant,
) {
    let source = Source::new();
    let mut files = source.write_files();
    let file = match eval {
        Some(code) => SourceFile {
            name: "eval".to_string(),
            source_code: SourceCode::String(code),
        },
        _ => panic!("No source code provided"),
    };

    files.push(file);

    let file = files.first().unwrap();
    let lexer = Lexer::new(file.source_code.as_ref());
    let parser = Parser::new(SourceFileId(0), lexer);
    let ParseResult {
        syntax_tree,
        errors,
    } = parser.parse_main();
    let parse_time = start_time.elapsed();

    drop(files);

    if !errors.is_empty() {
        let dust_error = DustError::parse(errors, source);

        eprintln!("{}", dust_error.report());

        return;
    }

    if !no_output {
        println!("{syntax_tree}");
    }

    if time {
        print_times(&[("Parse Time", parse_time, None)]);
    }
}
