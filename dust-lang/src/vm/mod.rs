//! Virtual machine and errors
mod action;
mod call_frame;
mod thread;

use std::{rc::Rc, thread::Builder};

pub use call_frame::{CallFrame, Pointer, Register, RegisterTable, RuntimeValue};
pub use thread::Thread;

use crossbeam_channel::bounded;
use tracing::{span, Level};

use crate::{compile, Chunk, DustError, Value};

pub fn run(source: &str) -> Result<Option<Value>, DustError> {
    let chunk = compile(source)?;
    let vm = Vm::new(chunk);

    Ok(vm.run())
}

pub struct Vm {
    main_chunk: Chunk,
}

impl Vm {
    pub fn new(main_chunk: Chunk) -> Self {
        Self { main_chunk }
    }

    pub fn run(self) -> Option<Value> {
        let span = span!(Level::INFO, "Run");
        let _enter = span.enter();
        let thread_name = self
            .main_chunk
            .name
            .as_ref()
            .map(|name| name.to_string())
            .unwrap_or_else(|| "anonymous".to_string());
        let (tx, rx) = bounded(1);

        Builder::new()
            .name(thread_name)
            .spawn(move || {
                let main_chunk = Rc::new(self.main_chunk);
                let main_thread = Thread::new(main_chunk);
                let return_value = main_thread.run();
                let _ = tx.send(return_value);
            })
            .unwrap()
            .join()
            .unwrap();

        rx.recv().unwrap_or(None)
    }
}
