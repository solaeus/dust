use crate::{Lexer, Span};

use super::{CompileError, Compiler, DEFAULT_REGISTER_COUNT, Item, Module, Path};

const STD: &str = "
mod io {
    fn write_line(message: str) {
        write_line(message)
    }
}
";

pub fn generate_standard_library(dust_crate: &mut Module) -> Result<(), CompileError> {
    let mut std_crate = Module::new();
    let lexer = Lexer::new(STD);
    let mut compiler =
        Compiler::<DEFAULT_REGISTER_COUNT>::new_module(lexer, "std", &mut std_crate)?;
    let start = compiler.current_position.0;

    compiler.compile()?;

    let end = compiler.current_position.1;

    dust_crate.items.insert(
        Path::new_borrowed("std").unwrap(),
        (Item::Module(std_crate), Span(start, end)),
    );

    Ok(())
}
