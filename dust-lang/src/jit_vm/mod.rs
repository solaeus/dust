pub mod error;
mod ffi_functions;
mod jit_compiler;
mod object;
mod object_pool;
mod register;
#[cfg(test)]
mod tests;
pub mod thread_pool;

pub use error::JitError;
pub use jit_compiler::{JitCompiler, JitLogic};
pub use object::Object;
pub use object_pool::ObjectPool;
pub use register::{Register, RegisterTag};
pub use thread_pool::ThreadStatus;

use std::sync::Arc;

use tracing::error;

use crate::{
    compiler::Compiler,
    dust_crate::Program,
    dust_error::DustError,
    jit_vm::thread_pool::{ThreadMessage, ThreadPool},
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

const ERROR_REPLACEMENT_STR: &str = "<dust_vm_error>";

pub fn run_main(source_code: String) -> Result<Option<Value>, DustError> {
    let mut source = Source::new();

    source.add_file(SourceFile {
        name: "main.ds".to_string(),
        source_code: SourceCode::String(source_code),
    });

    let compiler = Compiler::new(source);
    let program = compiler.compile(Some("Dust Program".to_string()))?;
    let vm = JitVm::new(
        Arc::new(program),
        MINIMUM_OBJECT_HEAP_DEFAULT,
        MINIMUM_OBJECT_SWEEP_DEFAULT,
    );

    vm.run()
}

pub struct JitVm {
    thread_pool: ThreadPool,
}

impl JitVm {
    pub fn new(
        program: Arc<Program>,
        minimum_object_heap: usize,
        minimum_object_sweep: usize,
    ) -> Self {
        Self {
            thread_pool: ThreadPool::new(program, minimum_object_heap, minimum_object_sweep),
        }
    }

    pub fn run(self) -> Result<Option<Value>, DustError> {
        let spawner_clone = self.thread_pool.clone_spawner();

        self.thread_pool
            .lock_spawner()
            .spawn_named_thread("Dust Program".to_string(), 0, spawner_clone)
            .map_err(DustError::jit)?;

        let receiver = self.thread_pool.lock_spawner().clone_receiver();
        let mut return_result = None;

        while !self.thread_pool.lock_spawner().is_emply() {
            match receiver.recv() {
                Ok(ThreadMessage::Spawn {
                    thread_name,
                    prototype_index,
                }) => {
                    let spawner_clone = self.thread_pool.clone_spawner();

                    self.thread_pool
                        .lock_spawner()
                        .spawn_named_thread(thread_name, prototype_index, spawner_clone)
                        .map_err(DustError::jit)?;
                }
                Ok(ThreadMessage::Complete {
                    thread_id,
                    result,
                    prototype_index,
                }) => {
                    let result = result.map_err(DustError::jit)?;

                    if prototype_index == 0 {
                        return_result = result;
                    }

                    self.thread_pool
                        .lock_spawner()
                        .threads_mut()
                        .remove(&thread_id);
                }
                Err(error) => {
                    error!("JIT VM Thread Pool Error: {}", error);
                }
            }
        }

        Ok(return_result)
    }
}
