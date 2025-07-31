use tracing::{Level, span};

use crate::{Compiler, Module, Span};

use super::{Item, Path};

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
        _int_to_str(value)
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

pub(crate) fn generate_standard_library() -> Module {
    let logging = span!(Level::INFO, "std");
    let _span_guard = logging.enter();

    let mut compiler = Compiler::new();
    compiler.allow_native_functions = true;

    compiler
        .compile_library("std", STD)
        .expect("Failed to compile standard library")
}

pub fn apply_standard_library(module: &mut Module) {
    let std_crate = generate_standard_library();
    let std_position = Span(0, STD_LENGTH);

    module.items.insert(
        Path::new("std").unwrap(),
        (Item::Module(std_crate), std_position),
    );
}
