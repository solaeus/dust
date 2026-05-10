use std::io::{Write, stdout};

use dust_lang::{compiler::Compiler, disassembler::Disassembler};
use ron::{
    ser::PrettyConfig,
    ser::{to_string, to_string_pretty},
};

use crate::{
    build_source,
    cli::{CompileCommand, Output},
};

pub fn handle_compile_command(command: CompileCommand) {
    let CompileCommand {
        global: _,
        input,
        output,
    } = command;

    let source = build_source(input);
    let compiler = Compiler::new(source);
    let (program, source, syntax, constants) = match compiler.compile_with_extras(None) {
        Ok(result) => result,
        Err(errors) => errors.print_and_exit(),
    };

    match output {
        Output::Tui => Disassembler::new(&program, &source, &syntax, &constants)
            .disassemble()
            .expect("Failed to run TUI disassembler"),
        Output::Debug => println!("{program:?}"),
        Output::Ron => {
            let ron_string = to_string(&program).expect("Failed to serialize program to RON");

            stdout().write_all(ron_string.as_bytes()).unwrap();
        }
        Output::PrettyRon => {
            let config = PrettyConfig::default().struct_names(true);
            let ron_string =
                to_string_pretty(&program, config).expect("Failed to serialize program to RON");

            stdout().write_all(ron_string.as_bytes()).unwrap();
        }
    }
}
