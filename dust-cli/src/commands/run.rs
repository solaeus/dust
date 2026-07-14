use std::io::{Write, stdout};

use dust_compiler::compiler::Compiler;
use dust_vm::{Vm, VmConfig};

use crate::{build_source, cli::RunCommand, error::Error, get_name};

pub fn run<'src>(commmand: RunCommand) -> Result<(), Error<'src>> {
    let RunCommand {
        global: _,
        input,
        name,
    } = commmand;

    let name = get_name(name, &input);
    let source = build_source(input)?;
    let program = Compiler::new(source).compile(name)?;
    let vm = Vm::new(program, VmConfig::default());

    match vm.run() {
        Ok(Some(return_value)) => {
            let mut stdout = stdout().lock();

            stdout.write_all(return_value.to_string().as_bytes())?;
            stdout.write_all(b"\n")?;

            Ok(())
        }
        Ok(None) => Ok(()),
        Err(error) => Err(Error::DustVm(error)),
    }
}
