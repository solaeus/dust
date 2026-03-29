use dust_lang::{
    compiler::Compiler,
    vm::{MINIMUM_OBJECT_HEAP_DEFAULT, Vm},
};

use crate::{
    build_source,
    cli::{GlobalOptions, InputOptions, RunCommand},
};

pub fn handle_run_command(commmand: RunCommand) {
    let RunCommand {
        global: GlobalOptions { log, name },
        input: InputOptions { eval, stdin, path },
    } = commmand;

    let source = build_source(&eval, path, false);
    let compiler = Compiler::new(source);
    let compile_result = compiler.compile(None);

    let program = match compile_result {
        Ok(program) => program,
        Err(error) => {
            eprintln!("{error}");

            return;
        }
    };
    let jit_vm = Vm::new(
        program,
        MINIMUM_OBJECT_HEAP_DEFAULT,
        MINIMUM_OBJECT_HEAP_DEFAULT,
    );

    let run_result = jit_vm.run();
    let return_value = match run_result {
        Ok(return_value) => return_value,
        Err(error) => {
            eprintln!("{error}");

            return;
        }
    };

    if let Some(value) = return_value {
        println!("{value}");
    }
}
