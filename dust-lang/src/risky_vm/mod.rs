//! An experimental Dust virtual machine that uses `unsafe` code. This VM never emits errors.
//! Instead, errors are handled as panics in debug mode but, in release mode, the use of `unsafe`
//! will cause undefined behavior.
mod call_frame;
mod memory;
mod runners;
mod thread;

pub use call_frame::CallFrame;
pub use memory::{Memory, RegisterTable};
pub use runners::{RUNNERS, Runner};
pub use thread::Thread;

use std::{sync::Arc, thread::Builder};

use crossbeam_channel::bounded;
use tracing::{Level, span};

use crate::{Chunk, ConcreteValue, DustError, Value, compile};

pub fn run(source: &str) -> Result<Option<ConcreteValue>, DustError> {
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

    pub fn run(self) -> Option<ConcreteValue> {
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
                let main_chunk = Arc::new(self.main_chunk);
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
