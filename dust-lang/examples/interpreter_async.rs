use std::{
    sync::{mpsc::channel, Arc},
    thread,
    time::Duration,
};

use context::Context;
use dust_lang::*;

fn run_fibnacci(interpreter: &Interpreter, i: u8) -> Value {
    // These double brackets are not Dust syntax, it's just an escape sequence for Rust's format!
    // macro.
    let source = Arc::from(format!(
        "
        fib = fn (i: int) -> int {{
        	if i <= 1 {{
                i
            }} else {{
        		fib(i - 1) + fib(i - 2)
        	}}
        }}

        fib({i})"
    ));

    interpreter
        .run(Arc::from(i.to_string()), source)
        .unwrap() // Panic if there are errors.
        .unwrap() // Panic if the no value is returned.
}

fn main() {
    let interpreter = Interpreter::new(Context::new());
    let (tx, rx) = channel();

    for i in 1..10 {
        let interpreter = interpreter.clone();
        let tx = tx.clone();

        thread::spawn(move || {
            let value = run_fibnacci(&interpreter, i);

            tx.send(value).unwrap();
        });
    }

    // Give the threads half a second to finish.
    while let Ok(value) = rx.recv_timeout(Duration::from_millis(500)) {
        println!("{}", value);
    }
}
