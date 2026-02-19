use std::time::Instant;

use dust_lang::{compiler::Compiler, disassembler::Disassembler};

use crate::{
    cli::{CompileCommand, GlobalOptions, InputOptions, OutputOptions},
    handle_source, print_times,
};

pub fn handle_compile_command(command: CompileCommand, start_time: Instant) {
    let CompileCommand {
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
        tui,
    } = command;

    let source = handle_source(&eval, path, false);
    let compiler = Compiler::new(source);
    let compile_result = compiler.compile_with_extras(None);
    let compile_time = start_time.elapsed();
    let (program, source, syntax, resolver) = match compile_result {
        Ok(program_and_extras) => program_and_extras,
        Err(dust_error) => {
            if !no_output {
                eprintln!("{}", dust_error.report())
            }

            return;
        }
    };

    if !no_output {
        if tui {
            let disassembler = Disassembler::new(&program, &source, &syntax, &resolver);

            disassembler.disassemble().unwrap();
        } else {
            println!("{program:#?}");
        }
    }

    if time {
        print_times(&[("Compile Time", compile_time, None)]);
    }
}
