//! This VM never emits errors. Instead, errors are handled as panics.
mod call_frame;
mod cell;
mod jit;
mod object;
mod object_pool;
mod register;
pub mod thread;

pub use call_frame::CallFrame;
pub use cell::{Cell, CellValue};
pub use jit::{JIT_ERROR_TEXT, Jit, JitChunk, JitError};
pub use object::Object;
pub use object_pool::ObjectPool;
pub use register::Register;
pub use thread::{Thread, ThreadHandle, ThreadStatus};

use std::sync::{Arc, RwLock};

use crate::{DustError, Program, Value, compile};

pub type ThreadPool = Arc<RwLock<Vec<ThreadHandle>>>;

pub fn run(source: &'_ str) -> Result<Option<Value>, DustError<'_>> {
    let program = compile(source)?;
    let vm = JitVm::new();

    vm.run(program)
}

pub struct JitVm {
    thread_pool: ThreadPool,
}

impl JitVm {
    pub fn new() -> Self {
        let thread_pool = Arc::new(RwLock::new(Vec::with_capacity(1)));

        Self { thread_pool }
    }

    pub fn run<'src>(self, program: Program) -> Result<Option<Value>, DustError<'src>> {
        let mut chunks = program.prototypes;

        chunks.push(program.main_chunk);

        let mut cells = Vec::with_capacity(program.cell_count as usize);

        for _ in 0..program.cell_count {
            cells.push(Cell::default());
        }

        let cells = Arc::new(RwLock::new(cells));
        let main_thread = ThreadHandle::spawn(chunks, cells, Arc::clone(&self.thread_pool))
            .map_err(DustError::jit)?;
        let return_result = main_thread
            .handle
            .join()
            .expect("Main thread panicked")
            .map_err(DustError::jit)?;
        let mut threads = self.thread_pool.write().expect("Failed to lock threads");

        for thread_handle in threads.drain(..) {
            thread_handle
                .handle
                .join()
                .expect("Thread panicked")
                .map_err(DustError::jit)?;
        }

        Ok(return_result)
    }
}

impl Default for JitVm {
    fn default() -> Self {
        Self::new()
    }
}
