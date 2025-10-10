#[macro_use]
mod call_stack;
mod cell;
mod jit_compiler;
mod object;
mod object_pool;
mod register;
pub mod thread;

pub use cell::{Cell, CellValue};
pub use jit_compiler::{JitCompiler, JitError, JitLogic};
pub use object::Object;
pub use object_pool::ObjectPool;
pub use register::{Register, RegisterTag};
pub use thread::{Thread, ThreadResult};

use std::sync::{Arc, RwLock};

use crate::{
    compiler::Compiler,
    dust_crate::Program,
    dust_error::DustError,
    source::{Source, SourceCode, SourceFile},
    value::Value,
};

pub const MINIMUM_OBJECT_HEAP_DEFAULT: usize = if cfg!(debug_assertions) {
    1024
} else {
    1024 * 1024 * 4
};
pub const MINIMUM_OBJECT_SWEEP_DEFAULT: usize = if cfg!(debug_assertions) {
    256
} else {
    1024 * 1024
};

pub type ThreadPool = Arc<RwLock<Vec<Thread>>>;

pub fn run_main(source_code: String) -> Result<Option<Value>, DustError> {
    let source = Source::new();

    source.add_file(SourceFile {
        name: "main.ds".to_string(),
        source_code: SourceCode::String(source_code),
    });

    let compiler = Compiler::new(source);
    let program = compiler.compile(Some("Dust Program".to_string()))?;
    let vm = JitVm::new();

    vm.run(
        Arc::new(program),
        MINIMUM_OBJECT_HEAP_DEFAULT,
        MINIMUM_OBJECT_SWEEP_DEFAULT,
    )
}

pub struct JitVm {
    thread_pool: ThreadPool,
}

impl JitVm {
    pub fn new() -> Self {
        let thread_pool = Arc::new(RwLock::new(Vec::with_capacity(1)));

        Self { thread_pool }
    }

    pub fn run(
        self,
        program: Arc<Program>,
        minimum_object_heap: usize,
        minimum_object_sweep: usize,
    ) -> Result<Option<Value>, DustError> {
        let main_thread = Thread::spawn(
            program.name.clone(),
            program,
            0,
            minimum_object_heap,
            minimum_object_sweep,
        )
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
