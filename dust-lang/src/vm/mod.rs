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
    DustError, StrippedChunk, Value, compile,
    dust_crate::Program,
    jit::{Jit, JitError},
};

pub type ThreadPool = Arc<RwLock<Vec<ThreadHandle>>>;

pub fn run(source: &'_ str) -> Result<Option<Value>, DustError<'_>> {
    let program = compile::<StrippedChunk>(source)?;
    let vm = Vm::new(program).map_err(DustError::jit)?;

    Ok(vm.run())
}

pub struct Vm {
    main_thread: ThreadHandle,
    threads: ThreadPool,
}

impl Vm {
    pub fn new(program: Program<StrippedChunk>) -> Result<Self, JitError> {
        let threads = Arc::new(RwLock::new(Vec::new()));
        let cells = Arc::new(RwLock::new(Vec::<Cell>::new()));

        let mut jit = Jit::new();
        let mut jit_chunks = Vec::new();

        for chunk in program.prototypes {
            let jit_chunk = jit.compile(&chunk)?;

            jit_chunks.push(jit_chunk);
        }

        let main_chunk = jit.compile(&program.main)?;
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
