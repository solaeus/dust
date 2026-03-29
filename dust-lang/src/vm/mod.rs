mod call_frame;
pub mod error;
mod object;
mod object_pool;
mod register;
mod thread;
mod thread_pool;

use std::sync::Arc;

use tracing::{Level, error, info, span};

use crate::{
    compiler::Compiler,
    dust_value::DustValue,
    error::{Error, ErrorKind},
    program::Program,
    source::{Source, SourceFile},
    vm::thread_pool::{ThreadMessage, ThreadPool},
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

pub fn run<'src>(source_code: &'src str) -> Result<Option<DustValue>, Error<'src>> {
    let mut source = Source::new();

    source.add_file(SourceFile::validated_borrowed("eval", source_code));

    let compiler = Compiler::new(source);
    let program = compiler.compile(None)?;
    let vm = Vm::new(
        Arc::new(program),
        MINIMUM_OBJECT_HEAP_DEFAULT,
        MINIMUM_OBJECT_SWEEP_DEFAULT,
    );

    vm.run()
}

pub struct Vm {
    thread_pool: ThreadPool,
}

impl Vm {
    pub fn new(
        program: Arc<Program>,
        minimum_object_heap: usize,
        minimum_object_sweep: usize,
    ) -> Self {
        Self {
            thread_pool: ThreadPool::new(program, minimum_object_heap, minimum_object_sweep),
        }
    }

    pub fn run<'a>(self) -> Result<Option<DustValue>, Error<'a>> {
        let span = span!(Level::INFO, "run");
        let _enter = span.enter();

        let message_receiver = {
            info!("Spawning main VM thread");

            let mut local_spawner = self.thread_pool.lock_spawner();

            local_spawner
                .spawn_thread(0)
                .map_err(|error| Error::without_context(vec![ErrorKind::Vm(error)]))?;
            local_spawner.clone_message_receiver()
        };

        let mut return_value: Option<DustValue> = None;

        loop {
            match message_receiver.recv() {
                Ok(ThreadMessage::SpawnThread {
                    thread_name,
                    prototype_id: prototype_index,
                }) => {
                    info!("Spawning VM thread: {thread_name} with proto_{prototype_index}");

                    self.thread_pool
                        .lock_spawner()
                        .spawn_named_thread(thread_name, prototype_index)
                        .map_err(|error| Error::without_context(vec![ErrorKind::Vm(error)]))?;
                }
                Ok(ThreadMessage::RemoveThread { thread_id, result }) => {
                    info!("VM thread completed: Thread ID: {}", thread_id.as_u64());

                    let return_registers = result
                        .map_err(|error| Error::without_context(vec![ErrorKind::Vm(error)]))?;

                    let mut spawner = self.thread_pool.lock_spawner();
                    let removed_handle = spawner.threads_mut().remove(&thread_id);

                    if let Some(handle) = removed_handle {
                        if handle.is_finished() {
                            info!(
                                "Thread {} was finished and removed from the thread pool.",
                                thread_id.as_u64()
                            );
                        } else {
                            error!(
                                "Thread {} was not finished when removed from the thread pool.",
                                thread_id.as_u64()
                            );
                        }
                    } else {
                        error!("Failed to remove VM thread.");
                    }

                    if spawner.is_empty() {
                        info!("All VM threads have completed.");

                        break;
                    }
                }
                Err(error) => {
                    error!("VM Error: {}", error);

                    break;
                }
            }
        }

        match return_value {
            Some(_) => todo!(),
            None => Ok(None),
        }
    }
}
