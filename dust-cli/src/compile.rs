use std::{path::PathBuf, time::Instant};

use dust_lang::{chunk::TuiDisassembler, compiler::Compiler};

use crate::{handle_source, print_times};

pub fn handle_compile_command(
    eval: Option<String>,
    path: Option<PathBuf>,
    no_output: bool,
    time: bool,
    start_time: Instant,
) {
    let source = handle_source(eval, path, false);
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
