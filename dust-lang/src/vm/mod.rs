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

use tracing::{Level, span};

use crate::{DustError, Program, Value, compile};

pub type ThreadPool = Arc<RwLock<Vec<ThreadHandle>>>;

pub fn run(source: &'_ str) -> Result<Option<Value>, DustError<'_>> {
    let chunk = compile(source)?;
    let vm = Vm::new(chunk);

    vm.run()
}

pub struct Vm {
    main_thread: ThreadHandle,
    threads: ThreadPool,
}

impl Vm {
    pub fn new(program: Program) -> Self {
        let threads = Arc::new(RwLock::new(Vec::new()));

        let mut cells = Vec::with_capacity(program.cell_count as usize);

        for _ in 0..program.cell_count {
            cells.push(Cell::default());
        }

        let cells = Arc::new(RwLock::new(cells));
        let main_thread = ThreadHandle::new(program.main_chunk, cells, Arc::clone(&threads));

        Self {
            main_thread,
            threads,
        }
    }

    pub fn run<'src>(self) -> Result<Option<Value>, DustError<'src>> {
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
            Err(DustError::jit(error))
        } else {
            match return_result {
                Ok(value_option) => Ok(value_option),
                Err(error) => Err(DustError::jit(error)),
            }
        }
    }
}
