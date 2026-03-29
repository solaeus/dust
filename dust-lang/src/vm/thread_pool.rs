use std::{
    collections::HashMap,
    sync::{Arc, Mutex, MutexGuard},
    thread::{self, Builder as ThreadBuilder, JoinHandle, ThreadId},
};

use crossbeam_channel::{Receiver, Sender};
use rustc_hash::FxBuildHasher;

use crate::{
    dust_value::DustValue,
    program::Program,
    vm::{error::VmError, register::Register, thread::Thread},
};

pub struct ThreadPool {
    spawner: Arc<Mutex<ThreadSpawner>>,
}

impl ThreadPool {
    pub fn new(
        program: Arc<Program>,
        minimum_object_heap: usize,
        minimum_object_sweep: usize,
    ) -> Self {
        let (sender, receiver) = crossbeam_channel::unbounded();

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

    pub fn spawn_thread(&mut self, prototype_id: u16) -> Result<(), VmError> {
        let message_sender = Arc::clone(&self.message_sender);
        let program = Arc::clone(&self.program);
        let minimum_object_heap = self.minimum_object_heap;
        let minimum_object_sweep = self.minimum_object_sweep;
        let join_handle = ThreadBuilder::new()
            .spawn(move || {
                let thread = Thread::new(program, prototype_id, message_sender);

                thread.run();
            })
            .expect("Failed to spawn thread");

        self.threads.insert(join_handle.thread().id(), join_handle);

        Ok(())
    }

    pub fn spawn_named_thread(
        &mut self,
        thread_name: String,
        prototype_id: u16,
    ) -> Result<(), VmError> {
        let message_sender = Arc::clone(&self.message_sender);
        let program = Arc::clone(&self.program);
        let minimum_object_heap = self.minimum_object_heap;
        let minimum_object_sweep = self.minimum_object_sweep;
        let join_handle = ThreadBuilder::new()
            .name(thread_name)
            .spawn(move || {
                let thread = Thread::new(program, prototype_id, message_sender);

                thread.run();
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

pub enum ThreadMessage {
    SpawnThread {
        thread_name: String,
        prototype_id: u16,
    },
    RemoveThread {
        thread_id: ThreadId,
        result: Result<Vec<Register>, VmError>,
    },
}

#[repr(C)]
pub struct ThreadStatus(u8);

impl ThreadStatus {
    pub const OK: Self = Self(0);
    pub const ERROR_LIST_INDEX_OUT_OF_BOUNDS: Self = Self(1);
    pub const ERROR_DIVISION_BY_ZERO: Self = Self(2);
}
