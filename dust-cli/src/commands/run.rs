use std::sync::Arc;

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
    let vm = Vm::new(Arc::new(program), VmConfig::default());

    match vm.run() {
        Ok(Some(return_value)) => {
            println!("{return_value}");

            Ok(())
        }
        Ok(None) => Ok(()),
        Err(error) => Err(Error::DustVm(error)),
    }
}
