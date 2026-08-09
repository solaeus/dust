use std::{
    collections::HashMap,
    sync::{Arc, Mutex, MutexGuard},
    thread::{Builder as ThreadBuilder, JoinHandle, ThreadId, current_id as current_thread_id},
};

use crossbeam::channel::{self, Receiver, Sender};
use dust_compiler::program::Program;
use rustc_hash::FxBuildHasher;

use crate::{error::VmError, register::Register, thread::Thread};

pub struct ThreadPool {
    spawner: Arc<Mutex<ThreadSpawner>>,
}

impl ThreadPool {
    pub fn new(
        program: Arc<Program>,
        minimum_object_heap: usize,
        minimum_object_sweep: usize,
    ) -> Self {
        let (sender, receiver) = channel::unbounded();

        ThreadPool {
            spawner: Arc::new(Mutex::new(ThreadSpawner {
                program,
                threads: HashMap::default(),
                message_sender: Arc::new(sender),
                message_receiver: receiver,
                minimum_object_heap,
                minimum_object_sweep,
            })),
        }
    }

    pub fn lock_spawner(&self) -> MutexGuard<'_, ThreadSpawner> {
        self.spawner
            .lock()
            .expect("Failed to lock ThreadSpawner mutex")
    }
}

pub struct ThreadSpawner {
    program: Arc<Program>,

    threads: HashMap<ThreadId, JoinHandle<()>, FxBuildHasher>,

    message_sender: Arc<Sender<ThreadMessage>>,
    message_receiver: Receiver<ThreadMessage>,

    minimum_object_heap: usize,
    minimum_object_sweep: usize,
}

impl ThreadSpawner {
    pub fn is_empty(&self) -> bool {
        self.threads.is_empty()
    }

    pub fn spawn_thread(&mut self, prototype_id: u32) -> Result<(), VmError> {
        let message_sender = Arc::clone(&self.message_sender);
        let program = Arc::clone(&self.program);
        let _minimum_object_heap = self.minimum_object_heap;
        let _minimum_object_sweep = self.minimum_object_sweep;
        let join_handle = ThreadBuilder::new()
            .spawn(move || {
                Thread::new(program, prototype_id, Arc::clone(&message_sender)).run();
            })
            .expect("Failed to spawn thread");

        self.threads.insert(join_handle.thread().id(), join_handle);

        Ok(())
    }

    pub fn spawn_named_thread(
        &mut self,
        thread_name: String,
        prototype_id: u32,
    ) -> Result<(), VmError> {
        let message_sender = Arc::clone(&self.message_sender);
        let program = Arc::clone(&self.program);
        let _minimum_object_heap = self.minimum_object_heap;
        let _minimum_object_sweep = self.minimum_object_sweep;
        let join_handle = ThreadBuilder::new()
            .name(thread_name)
            .spawn(move || {
                Thread::new(program, prototype_id, Arc::clone(&message_sender)).run();
            })
            .expect("Failed to spawn thread");

        self.threads.insert(join_handle.thread().id(), join_handle);

        Ok(())
    }

    pub fn clone_message_receiver(&self) -> Receiver<ThreadMessage> {
        self.message_receiver.clone()
    }

    pub fn threads_mut(&mut self) -> &mut HashMap<ThreadId, JoinHandle<()>, FxBuildHasher> {
        &mut self.threads
    }
}

#[derive(Debug)]
pub enum ThreadMessage {
    SpawnThread {
        thread_name: String,
        prototype_index: u32,
    },
    ThreadFinished {
        thread_id: ThreadId,
        return_registers: Vec<Register>,
    },
    ThreadError {
        thread_id: ThreadId,
        error: VmError,
    },
}
