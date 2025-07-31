use std::{
    sync::{Arc, RwLock},
    thread::{Builder as ThreadBuilder, JoinHandle},
};

use tracing::{Level, info, span};

use crate::{
    Chunk, JitChunk, RunStatus, Value,
    instruction::{Call, OperandType},
};

use super::{
    CallFrame, Cell, ObjectPool, Register,
    jit::{Jit, JitError},
};

pub struct ThreadHandle {
    pub handle: JoinHandle<Result<Option<Value>, JitError>>,
}

impl ThreadHandle {
    pub fn spawn(
        chunks: Vec<Chunk>,
        cells: Arc<RwLock<Vec<Cell>>>,
        threads: Arc<RwLock<Vec<ThreadHandle>>>,
    ) -> Result<Self, JitError> {
        let name = chunks
            .last()
            .expect("No main chunk for thread")
            .name
            .as_ref()
            .map(|name| name.to_string())
            .unwrap_or_else(|| "anonymous".to_string());
        let mut object_pool = ObjectPool::new();

        info!("Spawning thread {name}");

        let handle = ThreadBuilder::new()
            .name(name)
            .spawn(move || {
                let jit_chunks = chunks
                    .iter()
                    .map(|chunk| Jit::new(chunk, &mut object_pool).compile())
                    .collect::<Result<Vec<_>, JitError>>()?;
                let thread_result = Thread {
                    object_pool: ObjectPool::new(),
                    threads,
                    cells,
                    return_value: None,
                }
                .run(jit_chunks);

                Ok(thread_result)
            })
            .expect("Failed to spawn thread");

        Ok(ThreadHandle { handle })
    }
}

#[repr(C)]
pub struct Thread {
    pub object_pool: ObjectPool,
    pub threads: Arc<RwLock<Vec<ThreadHandle>>>,
    pub cells: Arc<RwLock<Vec<Cell>>>,
    pub return_value: Option<Value>,
}

impl Thread {
    fn run(mut self, chunks: Vec<JitChunk>) -> Option<Value> {
        let span = span!(Level::INFO, "Thread");
        let _enter = span.enter();

        let main_chunk = chunks.last().expect("No main chunk for thread");

        let mut call_stack = Vec::new();
        let mut register_stack = vec![Register { empty: () }; main_chunk.register_tags.len()];

        let call = CallFrame::new(
            main_chunk,
            (0, main_chunk.register_tags.len()),
            0,
            OperandType::NONE,
        );

        call_stack.push(call);

        while let Some(mut call_frame) = call_stack.pop() {
            let logic = call_frame.jit_chunk.logic;
            let register_range = call_frame.register_range;
            let register_stack_window = &mut register_stack[register_range.0..register_range.1];
            let status = (logic)(
                &mut self,
                &mut call_frame,
                register_stack_window.as_mut_ptr(),
            );

            match status {
                RunStatus::Call => {
                    let Call {
                        destination,
                        function,
                        argument_count,
                        return_type,
                    } = Call::from(call_frame.next_call);
                    let register_range = (register_range.1, register_range.1 + argument_count);
                    let jit_chunk = chunks
                        .get(function.index)
                        .expect("Invalid destination index for call");
                    let next_call =
                        CallFrame::new(jit_chunk, register_range, destination.index, return_type);

                    call_stack.push(call_frame);
                    call_stack.push(next_call);
                }
                RunStatus::Return => todo!(),
            }
        }

        self.return_value
    }
}
