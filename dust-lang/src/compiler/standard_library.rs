use crate::{Lexer, Span};

use super::{CompileError, Compiler, DEFAULT_REGISTER_COUNT, Item, Module, Path};

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
    fn int_to_string(value: int) -> str {
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
    let mut std_crate = Module::new();
    let lexer = Lexer::new(STD);
    let mut compiler =
        Compiler::<DEFAULT_REGISTER_COUNT>::new_module(lexer, "std", &mut std_crate)?;

    compiler.allow_native_functions = true;

    let start = compiler.current_position.0;

    compiler.compile()?;

    let end = compiler.current_position.1;

    dust_crate.items.insert(
        Path::new_borrowed("std").unwrap(),
        (Item::Module(std_crate), Span(start, end)),
    );

    Ok(())
}
