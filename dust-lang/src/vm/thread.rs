use std::{sync::Arc, thread::JoinHandle};

use tracing::{info, trace};

use crate::{Chunk, DustString, Value};

use super::{Action, CallFrame};

pub struct Thread {
    chunk: Arc<Chunk>,
}

impl Thread {
    pub fn new(chunk: Arc<Chunk>) -> Self {
        Thread { chunk }
    }

    pub fn run(&mut self) -> Option<Value> {
        info!(
            "Starting thread with {}",
            self.chunk
                .name
                .clone()
                .unwrap_or_else(|| DustString::from("anonymous"))
        );

        let mut call_stack = Vec::with_capacity(self.chunk.prototypes.len() + 1);
        let mut main_call = CallFrame::new(self.chunk.clone(), 0);

        call_stack.push(main_call);

        let mut thread_data = ThreadData {
            stack: call_stack,
            return_value: None,
            spawned_threads: Vec::with_capacity(0),
        };
        let mut action_sequence = Vec::with_capacity(self.chunk.instructions.len());

        for instruction in self.chunk.instructions.iter() {
            action_sequence.push(Action::from(instruction));
        }

        loop {
            let current_frame = thread_data.current_frame_mut();
            let current_action = if cfg!(debug_assertions) {
                action_sequence
                    .get(current_frame.instruction_pointer)
                    .unwrap()
            } else {
                unsafe {
                    action_sequence
                        .get(current_frame.instruction_pointer)
                        .unwrap_unchecked()
                }
            };
            current_frame.instruction_pointer += 1;

            trace!("Instruction: {}", current_action.fields);

            (current_action.logic)(current_action.fields, &mut thread_data);

            if let Some(value_option) = thread_data.return_value {
                return value_option;
            }
        }
    }
}

#[derive(Debug)]
pub struct ThreadData {
    pub stack: Vec<CallFrame>,
    pub return_value: Option<Option<Value>>,
    pub spawned_threads: Vec<JoinHandle<()>>,
}

impl ThreadData {
    pub fn current_frame(&self) -> &CallFrame {
        if cfg!(debug_assertions) {
            self.stack.last().unwrap()
        } else {
            unsafe { self.stack.last().unwrap_unchecked() }
        }
    }

    pub fn current_frame_mut(&mut self) -> &mut CallFrame {
        if cfg!(debug_assertions) {
            self.stack.last_mut().unwrap()
        } else {
            unsafe { self.stack.last_mut().unwrap_unchecked() }
        }
    }
}
