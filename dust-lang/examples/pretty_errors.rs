// It's very easy to get nice-looking error messages from the Dust's top-level error type.

use std::{io::stderr, sync::Arc};

use ariadne::sources;
use dust_lang::Interpreter;

fn main() {
    let interpreter = Interpreter::new();

    // First, we'll run some bad code.
    let error = interpreter
        .run(
            Arc::from("bad code"),
            Arc::from(
                "
            x = 1 + 'a'
            y: float = 'hello'
        ",
            ),
        )
        .unwrap_err();

    for report in error.build_reports() {
        report
            .write_for_stdout(sources(interpreter.sources()), stderr())
            .unwrap();
    }
}
