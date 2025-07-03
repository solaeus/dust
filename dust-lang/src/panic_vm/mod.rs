//! This VM never emits errors. Instead, errors are handled as panics.
mod call_frame;
mod cell;
pub mod macros;
mod memory;
mod thread;

pub use call_frame::CallFrame;
pub use cell::{Cell, CellValue};
pub use memory::{Memory, Object, Register};
pub use thread::Thread;

use std::sync::{Arc, RwLock};

use tracing::{Level, span};

use crate::{Chunk, DustError, StrippedChunk, Value, compile};

pub type ThreadPool<C> = Arc<RwLock<Vec<Thread<C>>>>;

pub fn run(source: &'_ str) -> Result<Option<Value<StrippedChunk>>, DustError<'_>> {
    let chunk = compile::<StrippedChunk>(source)?;
    let vm = Vm::new(chunk);

    Ok(vm.run())
}

pub struct Vm<C> {
    main_thread: Thread<C>,
    threads: ThreadPool<C>,
}

impl<C> Vm<C>
where
    C: 'static + Chunk + Send + Sync,
{
    pub fn new(main_chunk: C) -> Self {
        let threads = Arc::new(RwLock::new(Vec::new()));
        let cells = Arc::new(RwLock::new(Vec::<Cell<C>>::new()));
        let main_thread = Thread::new(main_chunk, cells, Arc::clone(&threads));

        Self {
            main_thread,
            threads,
        }
    }

    pub fn run(self) -> Option<Value<C>> {
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
