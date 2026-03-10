use dust_lang::{compiler::Compiler, disassembler::Disassembler};

use crate::{
    build_source,
    cli::{CompileCommand, GlobalOptions, InputOptions, OutputOptions},
};

pub fn handle_compile_command(command: CompileCommand) {
    let CompileCommand {
        global: GlobalOptions { log: _, name: _ },
        input: InputOptions { eval, stdin, path },
        output:
            OutputOptions {
                ron: _,
                pretty_ron: _,
                postcard: _,
            },
        tui,
    } = command;

    let source = build_source(&eval, path, stdin);
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
