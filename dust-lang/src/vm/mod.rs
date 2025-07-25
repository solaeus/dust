//! This VM never emits errors. Instead, errors are handled as panics.
mod call_frame;
mod cell;
mod memory;
pub mod thread;

pub use call_frame::CallFrame;
pub use cell::{Cell, CellValue};
pub use memory::{Object, Register};
pub use thread::{Thread, ThreadHandle};

use std::sync::{Arc, RwLock};

use tracing::{Level, span};

use crate::{
    DustError, Value, compile,
    dust_crate::Program,
    jit::{Jit, JitError},
};

pub type ThreadPool = Arc<RwLock<Vec<ThreadHandle>>>;

pub fn run(source: &'_ str) -> Result<Option<Value>, DustError<'_>> {
    let program = compile(source)?;
    let vm = Vm::new(program).map_err(DustError::jit)?;

    Ok(vm.run())
}

pub struct Vm {
    main_thread: ThreadHandle,
    threads: ThreadPool,
}

impl Vm {
    pub fn new(program: Program) -> Result<Self, JitError> {
        let threads = Arc::new(RwLock::new(Vec::new()));
        let cells = Arc::new(RwLock::new(Vec::<Cell>::new()));
        let mut jit = Jit::new()?;
        let (main_chunk, jit_chunks) = jit.compile(program)?;
        let main_thread = ThreadHandle::new(
            main_chunk,
            Arc::new(jit_chunks),
            cells,
            Arc::clone(&threads),
        );

        Ok(Self {
            main_thread,
            threads,
        })
    }

    pub fn run(self) -> Option<Value> {
        let span = span!(Level::INFO, "Run");
        let _enter = span.enter();

        let return_value = self
            .main_thread
            .handle
            .join()
            .expect("Main thread panicked");
        let mut threads = self.threads.write().expect("Failed to lock threads");

        for thread in threads.drain(..) {
            let _ = thread.handle.join().expect("Thread panicked");
        }

        return_value
    }
}
