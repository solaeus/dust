//! This VM never emits errors. Instead, errors are handled as panics.
mod call_frame;
pub mod macros;
mod memory;
mod thread;

pub use call_frame::CallFrame;
pub use memory::{Memory, RegisterTable};
pub use thread::Thread;

use std::sync::{Arc, Mutex};

use tracing::{Level, span};

use crate::{Chunk, ConcreteValue, DustError, Value, compile, compiler::DEFAULT_REGISTER_COUNT};

pub type ThreadPool<const REGISTER_COUNT: usize> = Arc<Mutex<Vec<Thread<REGISTER_COUNT>>>>;

pub fn run(source: &str) -> Result<Option<Value>, DustError> {
    let chunk = compile(source)?;
    let mut vm = Vm::<DEFAULT_REGISTER_COUNT>::new(Arc::new(chunk));

    Ok(vm.run().map(Value::Concrete))
}

pub struct Vm<const REGISTER_COUNT: usize> {
    main_chunk: Arc<Chunk>,
    threads: ThreadPool<REGISTER_COUNT>,
}

impl<const REGISTER_COUNT: usize> Vm<REGISTER_COUNT> {
    pub fn new(main_chunk: Arc<Chunk>) -> Self {
        Self {
            main_chunk,
            threads: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn run(&mut self) -> Option<ConcreteValue> {
        let span = span!(Level::INFO, "Run");
        let _enter = span.enter();

        let mut threads = self.threads.lock().expect("Failed to lock threads");

        let main_thread =
            Thread::<REGISTER_COUNT>::new(Arc::clone(&self.main_chunk), Arc::clone(&self.threads));

        threads.push(main_thread);

        let mut return_value = None;

        for thread in threads.drain(..) {
            return_value = thread.handle.join().expect("Thread panicked");
        }

        return_value
    }
}
