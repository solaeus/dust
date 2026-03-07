use dust_lang::prelude::*;

use crate::{
    cli::{CompileCommand, GlobalOptions, InputOptions, OutputOptions},
    handle_source,
};

pub fn handle_compile_command(command: CompileCommand) {
    let CompileCommand {
        global: GlobalOptions { log: _, name: _ },
        input: InputOptions {
            eval,
            stdin: _,
            path,
        },
        output:
            OutputOptions {
                ron: _,
                pretty_ron: _,
                postcard: _,
            },
        tui,
    } = command;

    let source = match handle_source(&eval, path, false) {
        Ok(source) => source,
        Err(error) => error.print_and_exit(),
    };
    let compiler = Compiler::new(source);
    let (program, source, syntax, resolver, constants) = match compiler.compile_with_extras(None) {
        Ok(result) => result,
        Err(errors) => errors.print_and_exit(),
    };

    if tui {
        let disassembler = Disassembler::new(&program, &source, &syntax, &resolver, &constants);

        disassembler.disassemble().unwrap();
    } else {
        println!("{program:#?}");
    }
}
