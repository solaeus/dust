use std::{
    sync::{Arc, RwLock},
    thread::{Builder as ThreadBuilder, JoinHandle},
};

use tracing::{Level, info, span, trace};

use crate::{
    Chunk, Instruction, JitExecutor, Program, Value,
    instruction::{Call, OperandType},
    jit_vm::call_frame::CallFrame,
};

use super::{
    Cell, ObjectPool, Register,
    jit::{Jit, JitError},
};

pub struct ThreadHandle {
    pub handle: JoinHandle<Result<Option<Value>, JitError>>,
}

impl ThreadHandle {
    pub fn spawn(
        program: Program,
        cells: Arc<RwLock<Vec<Cell>>>,
        threads: Arc<RwLock<Vec<ThreadHandle>>>,
    ) -> Result<Self, JitError> {
        let name = program
            .main_chunk
            .name
            .as_ref()
            .map(|name| name.to_string())
            .unwrap_or_else(|| "anonymous".to_string());

        info!("Spawning thread {name}");

        let handle = ThreadBuilder::new()
            .name(name)
            .spawn(move || {
                let mut jit = Jit::new(&program);
                let mut call_stack = Vec::new();
                let mut register_stack =
                    Vec::with_capacity(program.main_chunk.register_tags.len() as usize);
                let mut object_pool = ObjectPool::new();
                let mut thread = Thread {
                    register_stack: register_stack.as_mut_ptr(),
                    call_stack: call_stack.as_mut_ptr(),
                    object_pool: &mut object_pool,
                    threads: &threads,
                    cells: &cells,
                    return_value: None,
                };

                let jit_executor = jit.compile()?;

                (jit_executor)(&mut thread);

                Ok(thread.return_value)
            })
            .expect("Failed to spawn thread");

        Ok(ThreadHandle { handle })
    }
}

#[repr(C)]
pub struct Thread {
    pub register_stack: *mut Register,
    pub call_stack: *mut CallFrame,
    pub object_pool: *mut ObjectPool,
    pub threads: *const Arc<RwLock<Vec<ThreadHandle>>>,
    pub cells: *const Arc<RwLock<Vec<Cell>>>,
    pub return_value: Option<Value>,
}

#[repr(C)]
pub enum ThreadStatus {
    Call = 0,
    Return = 1,
}

pub fn read_field<T: Copy>(frame: &[u8], offset: usize) -> T {
    assert!(offset + size_of::<T>() <= frame.len());
    unsafe { *(frame.as_ptr().add(offset) as *const T) }
}

pub fn write_field<T: Copy>(frame: &mut [u8], offset: usize, value: T) {
    assert!(offset + size_of::<T>() <= frame.len());
    unsafe {
        *(frame.as_mut_ptr().add(offset) as *mut T) = value;
    }
}
