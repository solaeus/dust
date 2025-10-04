use std::time::Instant;

use dust_lang::{
    chunk::TuiDisassembler,
    compiler::Compiler,
    dust_error::DustError,
    lexer::Lexer,
    parser::{ParseResult, Parser},
    source::{Source, SourceCode, SourceFile, SourceFileId},
};

use crate::print_times;

pub fn handle_compile_command(
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
    } = parser.parse();
    let parse_time = start_time.elapsed();

    drop(files);

    if !errors.is_empty() {
        let dust_error = DustError::parse(errors, source);

        eprintln!("{}", dust_error.report());

        return;
    }

    let compiler = Compiler::new(source.clone());
    let compile_result = compiler.compile();
    let compile_time = start_time.elapsed() - parse_time;

    let (program, resolver) = match compile_result {
        Ok((program, resolver)) => (program, resolver),
        Err(dust_error) => {
            if !no_output {
                eprintln!("{}", dust_error.report())
            }

            return;
        }
    };
    let syntax_trees = vec![syntax_tree];

    if !no_output {
        let disassembler = TuiDisassembler::new(&program, &source, &syntax_trees, &resolver);

        disassembler.disassemble().unwrap();
    }

    if time {
        print_times(&[("Compile Time", compile_time, None)]);
    }
}
