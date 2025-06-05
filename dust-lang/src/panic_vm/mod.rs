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
    let vm = Vm::<DEFAULT_REGISTER_COUNT>::new(Arc::new(chunk));

    Ok(vm.run().map(Value::Concrete))
}

pub struct Vm<const REGISTER_COUNT: usize> {
    main_thread: Thread<REGISTER_COUNT>,
    threads: ThreadPool<REGISTER_COUNT>,
}

impl<const REGISTER_COUNT: usize> Vm<REGISTER_COUNT> {
    pub fn new(main_chunk: Arc<Chunk>) -> Self {
        let threads = Arc::new(Mutex::new(Vec::new()));
        let main_thread = Thread::<REGISTER_COUNT>::new(main_chunk, Arc::clone(&threads));

        Self {
            main_thread,
            threads,
        }
    }

    pub fn run(self) -> Option<ConcreteValue> {
        let span = span!(Level::INFO, "Run");
        let _enter = span.enter();

        let return_result = self
            .main_thread
            .handle
            .join()
            .expect("Main thread panicked");
        let mut threads = self.threads.lock().expect("Failed to lock threads");

        for thread in threads.drain(..) {
            thread.handle.join().expect("Thread panicked");
        }

        return_result
    }
}
