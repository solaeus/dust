use std::io::{Write, stdout};

use dust_lang::{
    compiler::Compiler,
    vm::{MINIMUM_OBJECT_HEAP_DEFAULT, Vm},
};

use crate::{build_source, cli::RunCommand, error::Error};

pub fn run<'src>(commmand: RunCommand) -> Result<(), Error<'src>> {
    let RunCommand { global: _, input } = commmand;

    let source = build_source(input)?;
    let compiler = Compiler::new(source);
    let compile_result = compiler.compile(None);

    let program = match compile_result {
        Ok(program) => program,
        Err(error) => return Err(Error::Dust(error)),
    };
    let jit_vm = Vm::new(
        program,
        MINIMUM_OBJECT_HEAP_DEFAULT,
        MINIMUM_OBJECT_HEAP_DEFAULT,
    );

    match jit_vm.run() {
        Ok(Some(return_value)) => {
            stdout().write_all(return_value.to_string().as_bytes())?;

            Ok(())
        }
        Ok(None) => Ok(()),
        Err(error) => Err(Error::Dust(error)),
    }
}
