use std::{
    sync::{Arc, RwLock},
    thread::{Builder as ThreadBuilder, JoinHandle},
};

use tracing::{Level, info, span, trace};

use crate::{
    Chunk, Instruction, JitChunk, Value,
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

                info!("Compiled {} JIT chunks", jit_chunks.len());

                let thread_result = Thread {
                    object_pool: ObjectPool::new(),
                    threads,
                    cells,
                    return_value: Register { empty: () },
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
    pub return_value: Register,
}

impl Thread {
    fn run(mut self, chunks: Vec<JitChunk>) -> Option<Value> {
        let span = span!(Level::INFO, "Thread");
        let _enter = span.enter();

        let main_chunk = chunks.last().expect("No main chunk for thread");

        let mut call_stack = Vec::new();
        let mut register_stack = vec![Register { empty: () }; main_chunk.register_tags.len()];

        let main_call = CallFrame::new(
            main_chunk,
            &chunks,
            (0, main_chunk.register_tags.len()),
            main_chunk.return_type,
            0,
        );

        call_stack.push(main_call);

        while let Some(mut current_call) = call_stack.pop() {
            let logic = current_call.jit_chunk.logic;
            let register_range = current_call.register_range;
            let register_stack_window = &mut register_stack[register_range.0..register_range.1];
            let status = (logic)(
                &mut self,
                &mut current_call,
                register_stack_window.as_mut_ptr(),
            );

            match status {
                ThreadStatus::Call => {
                    let next_call_instruction =
                        Instruction(current_call.next_call_instruction as u64);
                    let Call {
                        destination,
                        prototype_index,
                        arguments_index,
                        return_type,
                    } = Call::from(next_call_instruction);

                    let new_jit_chunk = chunks
                        .get(prototype_index)
                        .expect("Invalid destination index for call");
                    let start_register = register_range.1;
                    let end_register = start_register + new_jit_chunk.register_tags.len();

                    register_stack.resize(end_register, Register { empty: () });

                    let arguments = current_call
                        .jit_chunk
                        .argument_lists
                        .get(arguments_index)
                        .expect("Invalid arguments index for call");

                    for ((address, r#type), destination_index) in
                        arguments.iter().zip(start_register..)
                    {
                        match *r#type {
                            OperandType::INTEGER => {
                                let register =
                                    register_stack[current_call.register_range.0 + address.index];

                                register_stack[destination_index] = register;
                            }
                            _ => todo!("Unsupported argument: {}", address.to_string(*r#type)),
                        }
                    }

                    let next_call = CallFrame::new(
                        new_jit_chunk,
                        &chunks,
                        (start_register, end_register),
                        return_type,
                        destination.index,
                    );

                    trace!("Calling function proto_{prototype_index}");

                    call_stack.push(current_call);
                    call_stack.push(next_call);
                }
                ThreadStatus::Return => {
                    trace!("Returning from function");

                    let CallFrame {
                        return_type,
                        return_register_index,
                        ..
                    } = current_call;

                    if call_stack.is_empty() {
                        match return_type {
                            OperandType::NONE => return None,
                            OperandType::INTEGER => {
                                let integer = unsafe { self.return_value.integer };

                                return Some(Value::Integer(integer));
                            }
                            _ => todo!(),
                        }
                    } else {
                        match return_type {
                            OperandType::INTEGER => {
                                register_stack[return_register_index] = self.return_value;
                            }
                            _ => todo!(),
                        }
                    }
                }
            }
        }

        None
    }
}

#[repr(C)]
pub enum ThreadStatus {
    Call = 0,
    Return = 1,
}
