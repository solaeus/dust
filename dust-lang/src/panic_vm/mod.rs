//! This VM never emits errors. Instead, errors are handled as panics.
mod call_frame;
pub mod macros;
mod memory;
mod thread;

pub use call_frame::CallFrame;
pub use memory::{Memory, RegisterTable};
pub use thread::Thread;

use std::sync::{Arc, RwLock};

use tracing::{Level, span};

use crate::{Chunk, ConcreteValue, DustError, Value, compile};

pub type ThreadPool = Arc<RwLock<Vec<Thread>>>;

pub fn run(source: &str) -> Result<Option<Value>, DustError> {
    let chunk = compile(source)?;
    let vm = Vm::new(Arc::new(chunk));

    Ok(vm.run().map(Value::Concrete))
}

pub struct Vm {
    main_thread: Thread,
    threads: ThreadPool,
}

impl Vm {
    pub fn new(main_chunk: Arc<Chunk>) -> Self {
        let threads = Arc::new(RwLock::new(Vec::new()));
        let main_thread = Thread::new(main_chunk, Arc::clone(&threads));

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
        let mut threads = self.threads.write().expect("Failed to lock threads");

        for thread in threads.drain(..) {
            thread.handle.join().expect("Thread panicked");
        }

        return_result
    }
}
