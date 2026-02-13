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
use jit_compiler::{JitCompiler, JitFunction};
use object::Object;
use object_pool::ObjectPool;
use register::{Register, RegisterTag};
use thread_pool::ThreadStatus;

use std::sync::Arc;

use tracing::{Level, error, info, span};

use crate::{
    compiler::Compiler,
    dust_crate::Program,
    dust_error::DustError,
    jit_vm::thread_pool::{ThreadMessage, ThreadPool},
    source::{Source, SourceFile},
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

const STRING_ERROR_TEXT: &str = "Expected string object";

pub fn run_main<'a>(source_code: &'a str) -> Result<Option<Value>, DustError<'a>> {
    let mut source = Source::new();

    source.add_file(SourceFile::embedded_validated("eval", source_code));

    let compiler = Compiler::new(source);
    let program = compiler.compile(None)?;
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

    pub fn run<'a>(self) -> Result<Option<Value>, DustError<'a>> {
        let span = span!(Level::INFO, "jit_vm");
        let _enter = span.enter();

        let message_receiver = {
            info!("Spawning main JIT VM thread");

            let mut local_spawner = self.thread_pool.lock_spawner();
            let remote_spawner = self.thread_pool.clone_spawner();

            local_spawner
                .spawn_thread(0, remote_spawner)
                .map_err(DustError::jit)?;
            local_spawner.clone_message_receiver()
        };

        let mut return_result = None;

        loop {
            match message_receiver.recv() {
                Ok(ThreadMessage::SpawnThread {
                    thread_name,
                    prototype_id: prototype_index,
                }) => {
                    info!("Spawning JIT VM thread: {thread_name} with proto_{prototype_index}");

                    let spawner_clone = self.thread_pool.clone_spawner();

                    self.thread_pool
                        .lock_spawner()
                        .spawn_named_thread(thread_name, prototype_index, spawner_clone)
                        .map_err(DustError::jit)?;
                }
                Ok(ThreadMessage::RemoveThread {
                    thread_id,
                    result,
                    prototype_id,
                }) => {
                    info!("JIT VM thread completed: proto_{prototype_id}");

                    let result = result.map_err(DustError::jit)?;

                    if prototype_id == 0 {
                        return_result = result;
                    }

                    let removed_handle = self
                        .thread_pool
                        .lock_spawner()
                        .threads_mut()
                        .remove(&thread_id);

                    if let Some(handle) = removed_handle {
                        if handle.is_finished() {
                            info!(
                                "Thread {} running {prototype_id} was finished and removed from the thread pool.",
                                thread_id.as_u64()
                            );
                        } else {
                            error!(
                                "Thread {} running {prototype_id} was not finished when removed from the thread pool.",
                                thread_id.as_u64()
                            );
                        }
                    } else {
                        error!("Failed to remove JIT VM thread running {prototype_id}");
                    }

                    break;
                }
                Err(error) => {
                    error!("JIT VM Thread Pool Error: {}", error);

                    break;
                }
            }
        }

        Ok(return_result)
    }
}
