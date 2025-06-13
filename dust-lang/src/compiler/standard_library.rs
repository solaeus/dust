use std::collections::HashMap;

use tracing::{Level, span};

use crate::{Lexer, Span};

use super::{ChunkCompiler, CompileError, DEFAULT_REGISTER_COUNT, Item, Module, Path};

const STD: &str = r"
mod io {
    fn read_line() -> str {
        _read_line()
    }

    fn write_line(message: str) {
        _write_line(message)
    }
}

mod convert {
    fn int_to_str(value: int) -> str {
        _to_string(value)
    }
}

mod thread {
    fn spawn(function: fn()) {
        _spawn(function)
    }
}
";

pub fn generate_standard_library(dust_crate: &mut Module) -> Result<(), CompileError> {
    let logging = span!(Level::INFO, "Standard Library");
    let _span_guard = logging.enter();

    let mut std_module = Module::new();
    let mut globals = HashMap::new();
    let lexer = Lexer::new(STD);
    let mut compiler = ChunkCompiler::<DEFAULT_REGISTER_COUNT>::new_module(
        lexer,
        "std",
        &mut std_module,
        &mut globals,
    )?;

    compiler.allow_native_functions = true;

    let start = compiler.current_position.0;

    compiler.compile()?;

    let end = compiler.current_position.1;

    dust_crate.items.insert(
        Path::new_borrowed("std").unwrap(),
        (Item::Module(std_module), Span(start, end)),
    );

    Ok(())
}
