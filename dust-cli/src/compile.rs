use std::time::Instant;

use dust_lang::{
    chunk::TuiDisassembler,
    compiler::Compiler,
    source::{Source, SourceCode, SourceFile},
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

    drop(files);

    let compiler = Compiler::new(source.clone());
    let compile_result = compiler.compile();
    let compile_time = start_time.elapsed();

    let (program, resolver, syntax_trees) = match compile_result {
        Ok((program, resolver, syntax_trees)) => (program, resolver, syntax_trees),
        Err(dust_error) => {
            if !no_output {
                eprintln!("{}", dust_error.report())
            }

            return;
        }
    };

    if !no_output {
        let disassembler = TuiDisassembler::new(&program, &source, &syntax_trees, &resolver);

        disassembler.disassemble().unwrap();
    }

    if time {
        print_times(&[("Compile Time", compile_time, None)]);
    }
}
