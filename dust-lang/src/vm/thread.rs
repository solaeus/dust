use std::{sync::Arc, thread::JoinHandle};

use tracing::{info, trace};

use crate::{Chunk, DustString, Span, Value, vm::CallFrame};

use super::{Pointer, Register};

pub struct Thread {
    chunk: Arc<Chunk>,
    call_stack: Vec<CallFrame>,
    return_value_index: Option<Option<usize>>,
    spawned_threads: Vec<JoinHandle<()>>,
}

impl Thread {
    pub fn new(chunk: Arc<Chunk>) -> Self {
        let mut call_stack = Vec::with_capacity(chunk.prototypes.len() + 1);
        let main_call = CallFrame::new(Arc::clone(&chunk), 0);

        call_stack.push(main_call);

        Thread {
            chunk,
            call_stack,
            return_value_index: None,
            spawned_threads: Vec::new(),
        }
    }

    pub fn run(mut self) -> Option<Value> {
        info!(
            "Starting thread with {}",
            self.chunk
                .name
                .clone()
                .unwrap_or_else(|| DustString::from("anonymous"))
        );

        loop {
            let current_frame = self.current_frame_mut();
            let ip = {
                let ip = current_frame.ip;
                current_frame.ip += 1;

                ip
            };
            let current_action = if cfg!(debug_assertions) {
                current_frame.action_sequence.actions.get(ip).unwrap()
            } else {
                unsafe { current_frame.action_sequence.actions.get_unchecked(ip) }
            };

            trace!(
                "Instruction operation: {}",
                current_action.instruction.operation
            );

            (current_action.logic)(current_action.instruction, &mut self);

            if let Some(return_index_option) = self.return_value_index {
                if let Some(return_index) = return_index_option {
                    let return_value = self.get_register(return_index).clone();

                    return Some(return_value);
                } else {
                    return None;
                }
            }
        }
    }

    pub fn current_position(&self) -> Span {
        let current_frame = self.current_frame();

        current_frame.chunk.positions[current_frame.ip]
    }

    pub fn current_frame(&self) -> &CallFrame {
        if cfg!(debug_assertions) {
            self.call_stack.last().unwrap()
        } else {
            unsafe { self.call_stack.last().unwrap_unchecked() }
        }
    }

    pub fn current_frame_mut(&mut self) -> &mut CallFrame {
        if cfg!(debug_assertions) {
            self.call_stack.last_mut().unwrap()
        } else {
            unsafe { self.call_stack.last_mut().unwrap_unchecked() }
        }
    }

    pub fn get_register(&self, register_index: usize) -> &Value {
        trace!("Get R{register_index}");

        let register = if cfg!(debug_assertions) {
            self.call_stack
                .last()
                .unwrap()
                .registers
                .get(register_index)
                .unwrap()
        } else {
            unsafe {
                self.call_stack
                    .last()
                    .unwrap_unchecked()
                    .registers
                    .get_unchecked(register_index)
            }
        };

        match register {
            Register::Value(value) => value,
            Register::Pointer(pointer) => self.get_pointer_value(pointer),
            Register::Empty => panic!("Attempted to get value from empty register"),
        }
    }

    pub fn get_register_mut(&mut self, register_index: usize) -> &mut Register {
        trace!("Get R{register_index}");

        if cfg!(debug_assertions) {
            self.call_stack
                .last_mut()
                .unwrap()
                .registers
                .get_mut(register_index)
                .unwrap()
        } else {
            unsafe {
                self.call_stack
                    .last_mut()
                    .unwrap_unchecked()
                    .registers
                    .get_unchecked_mut(register_index)
            }
        }
    }

    pub fn get_constant(&self, constant_index: usize) -> &Value {
        if cfg!(debug_assertions) {
            self.chunk.constants.get(constant_index).unwrap()
        } else {
            unsafe { self.chunk.constants.get_unchecked(constant_index) }
        }
    }

    pub fn get_pointer_value(&self, pointer: &Pointer) -> &Value {
        match pointer {
            Pointer::Register(register_index) => self.get_register(*register_index),
            Pointer::Constant(constant_index) => self.get_constant(*constant_index),
            Pointer::Stack(call_index, register_index) => {
                let call_frame = if cfg!(debug_assertions) {
                    self.call_stack.get(*call_index).unwrap()
                } else {
                    unsafe { self.call_stack.get_unchecked(*call_index) }
                };
                let register = if cfg!(debug_assertions) {
                    call_frame.registers.get(*register_index).unwrap()
                } else {
                    unsafe { call_frame.registers.get_unchecked(*register_index) }
                };

                match register {
                    Register::Value(value) => value,
                    Register::Pointer(pointer) => self.get_pointer_value(pointer),
                    Register::Empty => panic!("Attempted to get value from empty register"),
                }
            }
        }
    }
}
