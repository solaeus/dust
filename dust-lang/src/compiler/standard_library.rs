use tracing::{Level, span};

use crate::{Chunk, Compiler, Span};

use super::{CompileError, Item, Module, Path};

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
        _int_to_string(value)
    }
}

mod thread {
    fn spawn(function: fn()) {
        _spawn(function)
    }
}
";

pub const STD_LENGTH: usize = {
    debug_assert!(STD.is_ascii(), "Standard library must be ASCII");

    STD.len()
};

pub(crate) fn generate_standard_library<'a, C: 'a + Chunk>() -> Result<Module<'a, C>, CompileError>
{
    let logging = span!(Level::INFO, "Standard Library");
    let _span_guard = logging.enter();

    let mut compiler = Compiler::new();
    compiler.allow_native_functions = true;

    let std_crate = compiler.compile_library("std", STD)?;

    Ok(std_crate)
}

pub fn apply_standard_library<'a, C: 'a + Chunk>(
    module: &mut Module<'a, C>,
) -> Result<(), CompileError> {
    let std_crate = generate_standard_library::<C>()?;
    let std_position = Span(0, STD_LENGTH);

    module.items.insert(
        Path::new_borrowed("std").unwrap(),
        (Item::Module(std_crate), std_position),
    );

    Ok(())
}
