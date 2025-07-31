//! This VM never emits errors. Instead, errors are handled as panics.
mod call_frame;
mod cell;
mod object;
mod object_pool;
mod register;
pub mod thread;

pub use call_frame::CallFrame;
pub use cell::{Cell, CellValue};
pub use object::Object;
pub use object_pool::ObjectPool;
pub use register::Register;
pub use thread::{Thread, ThreadHandle};

use std::sync::{Arc, RwLock};

use crate::{DustError, Program, Value, compile};

pub type ThreadPool = Arc<RwLock<Vec<ThreadHandle>>>;

pub fn run(source: &'_ str) -> Result<Option<Value>, DustError<'_>> {
    let program = compile(source)?;
    let vm = Vm::new();

    vm.run(program)
}

pub struct Vm {
    thread_pool: ThreadPool,
}

impl Vm {
    pub fn new() -> Self {
        let thread_pool = Arc::new(RwLock::new(Vec::with_capacity(1)));

        Self { thread_pool }
    }

    pub fn run<'src>(self, program: Program) -> Result<Option<Value>, DustError<'src>> {
        let mut cells = Vec::with_capacity(program.cell_count as usize);

        for _ in 0..program.cell_count {
            cells.push(Cell::default());
        }

        let cells = Arc::new(RwLock::new(cells));
        let main_thread =
            ThreadHandle::spawn(program.main_chunk, cells, Arc::clone(&self.thread_pool));

        let return_result = main_thread.handle.join().expect("Main thread panicked");
        let mut threads = self.thread_pool.write().expect("Failed to lock threads");

        for thread_handle in threads.drain(..) {
            thread_handle
                .handle
                .join()
                .expect("Thread panicked")
                .map_err(DustError::jit)?;
        }

        return_result.map_err(DustError::jit)
    }
}

impl Default for Vm {
    fn default() -> Self {
        Self::new()
    }
}
