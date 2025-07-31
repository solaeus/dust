use std::{
    sync::{Arc, RwLock},
    thread::{Builder as ThreadBuilder, JoinHandle},
};

use tracing::{Level, info, span};

use crate::{
    Chunk, Value,
    instruction::OperandType,
    jit::{Jit, JitError},
    vm::Register,
};

use super::{CallFrame, Cell, ObjectPool};

pub struct ThreadHandle {
    pub handle: JoinHandle<Result<Option<Value>, JitError>>,
}

impl ThreadHandle {
    pub fn spawn(
        chunk: Chunk,
        cells: Arc<RwLock<Vec<Cell>>>,
        threads: Arc<RwLock<Vec<ThreadHandle>>>,
    ) -> Self {
        let name = chunk
            .name
            .as_ref()
            .map(|name| name.to_string())
            .unwrap_or_else(|| "anonymous".to_string());
        let handle = ThreadBuilder::new()
            .name(name)
            .spawn(move || {
                Thread {
                    object_pool: ObjectPool::new(),
                    call_stack: Vec::new(),
                    threads,
                    cells,
                    return_value: None,
                }
                .run(chunk)
            })
            .expect("Failed to spawn thread");

        ThreadHandle { handle }
    }
}

#[repr(C)]
pub struct Thread {
    pub call_stack: Vec<CallFrame>,
    pub object_pool: ObjectPool,
    pub threads: Arc<RwLock<Vec<ThreadHandle>>>,
    pub cells: Arc<RwLock<Vec<Cell>>>,
    pub return_value: Option<Value>,
}

impl Thread {
    fn run(mut self, chunk: Chunk) -> Result<Option<Value>, JitError> {
        let span = span!(Level::INFO, "Thread");
        let _enter = span.enter();

        info!(
            "Starting thread {}",
            chunk
                .name
                .as_ref()
                .map(|name| name.as_ref())
                .unwrap_or_default()
        );

        let mut register_stack = vec![Register { empty: () }; chunk.register_tags.len()];
        let mut jit = Jit::new(&chunk, &mut self.object_pool);
        let jit_chunk = jit.compile()?;
        let call = CallFrame::new(
            jit_chunk,
            (0, chunk.register_tags.len()),
            true,
            0,
            OperandType::NONE,
        );

        self.call_stack.push(call);

        while let Some(mut call_frame) = self.call_stack.pop() {
            let logic = call_frame.jit_chunk.logic;
            let register_range = call_frame.register_range;
            let register_stack_window = &mut register_stack[register_range.0..register_range.1];

            (logic)(
                &mut self,
                &mut call_frame,
                register_stack_window.as_mut_ptr(),
            );

            if call_frame.push_back {
                self.call_stack.push(call_frame);
            }
        }

        Ok(self.return_value)
    }
}
