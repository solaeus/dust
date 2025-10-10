use std::{path::PathBuf, sync::Arc, time::Instant};

use dust_lang::{
    compiler::Compiler,
    jit_vm::{JitVm, MINIMUM_OBJECT_HEAP_DEFAULT},
};

use crate::{handle_source, print_times};

pub fn handle_run_command(
    eval: Option<String>,
    path: Option<PathBuf>,
    no_output: bool,
    time: bool,
    start_time: Instant,
) {
    let source = handle_source(eval, path, false);
    let compiler = Compiler::new(source.clone());
    let compile_result = compiler.compile(None);
    let compile_time = start_time.elapsed();
    let program = match compile_result {
        Ok(program) => program,
        Err(dust_error) => {
            if !no_output {
                eprintln!("{}", dust_error.report())
            }

            return;
        }
    };
    let jit_vm = JitVm::new();

    let run_result = jit_vm.run(
        Arc::new(program),
        MINIMUM_OBJECT_HEAP_DEFAULT,
        MINIMUM_OBJECT_HEAP_DEFAULT,
    );
    let run_time = start_time.elapsed() - compile_time;
    let return_value = match run_result {
        Ok(return_value) => return_value,
        Err(dust_error) => {
            if !no_output {
                eprintln!("{}", dust_error.report())
            }

            return;
        }
    };

    if let Some(value) = return_value
        && !no_output
    {
        println!("{value}");
    }

    if time {
        print_times(&[("Run Time", run_time, None)]);
    }
}
