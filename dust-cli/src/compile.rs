use std::io::{Write, stdout};

use dust_lang::{compiler::Compiler, disassembler::Disassembler};
use ron::ser::PrettyConfig;

use crate::{
    build_source,
    cli::{CompileCommand, InputOptions, OutputOptions},
};

pub fn handle_compile_command(command: CompileCommand) {
    let CompileCommand {
        global: _,
        input: InputOptions { eval, stdin, path },
        output:
            OutputOptions {
                debug,
                ron,
                pretty_ron,
                postcard,
            },
        tui,
    } = command;

    let source = build_source(&eval, path, stdin);
    let compiler = Compiler::new(source);
    let (program, source, syntax, constants) = match compiler.compile_with_extras(None) {
        Ok(result) => result,
        Err(errors) => errors.print_and_exit(),
    };

    if debug {
        println!("{program:#?}");
    } else if ron {
        let ron_string = ron::to_string(&program).expect("Failed to serialize program to RON");

        stdout()
            .write_all(ron_string.as_bytes())
            .expect("Failed to write RON output to stdout");
    } else if pretty_ron {
        let ron_string =
            ron::ser::to_string_pretty(&program, PrettyConfig::default().struct_names(true))
                .expect("Failed to serialize program to pretty RON");

        stdout()
            .write_all(ron_string.as_bytes())
            .expect("Failed to write pretty RON output to stdout");
    } else if postcard {
        let bytes = postcard::to_extend(&program, Vec::new())
            .expect("Failed to serialize program to Postcard");

        stdout()
            .write_all(&bytes)
            .expect("Failed to write Postcard output to stdout");
    } else if tui {
        let disassembler = Disassembler::new(&program, &source, &syntax, &constants);

        disassembler.disassemble().unwrap();
    }
}
