use std::io::{Write, stdout};

use dust_compiler::compiler::Compiler;
use ron::{
    ser::PrettyConfig,
    ser::{to_string, to_string_pretty},
};

use crate::{
    build_source,
    cli::{CompileCommand, CompileOutput},
    error::Error,
};

pub fn compile<'src>(command: CompileCommand) -> Result<(), Error<'src>> {
    let CompileCommand {
        global: _,
        input,
        output,
    } = command;

    let source = build_source(input)?;
    let compiler = Compiler::new(source);
    let program = compiler.compile()?;

    match output {
        CompileOutput::Debug => println!("{program:#?}"),
        CompileOutput::Ron => stdout().write_all(to_string(&program)?.as_bytes())?,
        CompileOutput::PrettyRon => {
            let config = PrettyConfig::default().struct_names(true);

            stdout().write_all(to_string_pretty(&program, config)?.as_bytes())?;
        }
    }

    Ok(())
}
