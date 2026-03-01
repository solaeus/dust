use std::time::Instant;

use dust_lang::prelude::*;

use crate::{
    cli::{CompileCommand, GlobalOptions, InputOptions, OutputOptions},
    handle_source, print_times,
};

pub fn handle_compile_command(
    command: CompileCommand,
    start_time: Instant,
) -> Result<(), ErrorKind> {
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

    let source = handle_source(&eval, path, false)?;
    let compiler = Compiler::new(source);
    let (program, source, syntax, resolver) = match compiler.compile_with_extras(None) {
        Ok(result) => result,
        Err(errors) => errors.print_and_exit(),
    };
    let compile_time = start_time.elapsed();

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

    Ok(())
}
