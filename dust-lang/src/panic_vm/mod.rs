//! This VM never emits errors. Instead, errors are handled as panics.
mod call_frame;
pub mod macros;
mod memory;
mod thread;

pub use call_frame::CallFrame;
pub use memory::{Memory, RegisterTable};
pub use thread::Thread;

use std::{sync::Arc, thread::Builder};

use crossbeam_channel::bounded;
use tracing::{Level, span};

use crate::{Chunk, ConcreteValue, DustError, Value, compile, compiler::DEFAULT_REGISTER_COUNT};

pub fn run(source: &str) -> Result<Option<Value>, DustError> {
    let chunk = compile(source)?;
    let vm = Vm::<DEFAULT_REGISTER_COUNT>::new(Arc::new(chunk));

    Ok(vm.run().map(Value::Concrete))
}

pub struct Vm<const REGISTER_COUNT: usize> {
    main_chunk: Arc<Chunk>,
}

impl<const REGISTER_COUNT: usize> Vm<REGISTER_COUNT> {
    pub fn new(main_chunk: Arc<Chunk>) -> Self {
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
        let _thread_result = Builder::new()
            .name(thread_name)
            .spawn(move || {
                let mut main_thread = Thread::<REGISTER_COUNT>::new(Arc::clone(&self.main_chunk));
                let return_value = main_thread.run();
                let _ = tx.send(return_value);
            })
            .expect("Failed to create the main thread.")
            .join();

        rx.recv().unwrap_or(None)
    }
}
