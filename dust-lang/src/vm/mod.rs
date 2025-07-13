//! This VM never emits errors. Instead, errors are handled as panics.
mod call_frame;
mod cell;
pub mod macros;
mod memory;
mod runtime_error;
mod thread;

pub use call_frame::CallFrame;
pub use cell::{Cell, CellValue};
pub use memory::{Object, Register};
pub use runtime_error::{RUNTIME_ERROR_TEXT, RuntimeError};
pub use thread::Thread;

use std::sync::{Arc, RwLock};

use tracing::{Level, span};

use crate::{DustError, StrippedChunk, Value, compile};

pub type ThreadPool = Arc<RwLock<Vec<Thread>>>;

pub fn run(source: &'_ str) -> Result<Option<Value<StrippedChunk>>, DustError<'_>> {
    let chunk = compile::<StrippedChunk>(source)?;
    let vm = Vm::new(chunk);

    vm.run()
}

pub struct Vm {
    main_thread: Thread,
    threads: ThreadPool,
}

impl Vm {
    pub fn new(main_chunk: StrippedChunk) -> Self {
        let threads = Arc::new(RwLock::new(Vec::new()));
        let cells = Arc::new(RwLock::new(Vec::<Cell>::new()));
        let main_thread = Thread::new(main_chunk, cells, Arc::clone(&threads));

        Self {
            main_thread,
            threads,
        }
    }

    pub fn run<'src>(self) -> Result<Option<Value<StrippedChunk>>, DustError<'src>> {
        let span = span!(Level::INFO, "Run");
        let _enter = span.enter();

        let return_result = self
            .main_thread
            .handle
            .join()
            .expect("Main thread panicked");
        let mut threads = self.threads.write().expect("Failed to lock threads");
        let mut spawned_thread_error = None;

        for thread in threads.drain(..) {
            let thread_result = thread.handle.join().expect("Thread panicked");

            if let Err(error) = thread_result {
                spawned_thread_error = Some(error);
            }
        }

        if let Some(error) = spawned_thread_error {
            Err(DustError::runtime(error))
        } else {
            match return_result {
                Ok(value_option) => Ok(value_option),
                Err(error) => Err(DustError::runtime(error)),
            }
        }
    }
}
