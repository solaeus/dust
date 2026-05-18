use std::io::{Write, stdout};

use dust_lang::{
    compiler::Compiler,
    vm::{MINIMUM_OBJECT_HEAP_DEFAULT, Vm},
};

use crate::{build_source, cli::RunCommand, error::Error};

pub fn run<'src>(commmand: RunCommand) -> Result<(), Error<'src>> {
    let RunCommand { global: _, input } = commmand;

    let source = build_source(input)?;
    let program = Compiler::new(source).compile(None)?;
    let vm = Vm::new(
        program,
        MINIMUM_OBJECT_HEAP_DEFAULT,
        MINIMUM_OBJECT_HEAP_DEFAULT,
    );

    match vm.run() {
        Ok(Some(return_value)) => {
            let mut stdout = stdout().lock();

            stdout.write_all(return_value.to_string().as_bytes())?;
            stdout.write_all(b"\n")?;

            Ok(())
        }
        Ok(None) => Ok(()),
        Err(error) => Err(Error::Dust(error)),
    }
}
