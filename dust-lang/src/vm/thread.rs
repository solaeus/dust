use std::{sync::Arc, thread::JoinHandle};

use smallvec::SmallVec;
use tracing::{info, span, trace};

use crate::{Chunk, DustString, Value, vm::action::ActionSequence};

use super::{Action, CallFrame};

pub struct Thread {
    chunk: Arc<Chunk>,
}

impl Thread {
    pub fn new(chunk: Arc<Chunk>) -> Self {
        Thread { chunk }
    }

    pub fn run(&mut self) -> Option<Value> {
        let span = span!(tracing::Level::INFO, "Thread");
        let _enter = span.enter();

        info!(
            "Starting thread with {}",
            self.chunk
                .name
                .clone()
                .unwrap_or_else(|| DustString::from("anonymous"))
        );

        let mut call_stack = SmallVec::with_capacity(self.chunk.prototypes.len() + 1);
        let main_call = CallFrame::new(Arc::clone(&self.chunk), 0);

        call_stack.push(main_call);

        let mut thread_data = ThreadData {
            stack: call_stack,
            return_value: None,
            spawned_threads: Vec::with_capacity(0),
        };
        let mut action_iter = self.chunk.instructions.iter().map(Action::from);
        let mut action_sequence = ActionSequence::new(&mut action_iter);

        trace!("Run thread main with actions: {:?}", action_sequence);

        loop {
            let current_frame = thread_data.current_frame_mut();
            let current_action = action_sequence.get_mut(current_frame.instruction_pointer);
            current_frame.instruction_pointer += 1;

            trace!("Action: {}", current_action);

            (current_action.logic)(&mut thread_data, &mut current_action.data);

            if let Some(value_option) = thread_data.return_value {
                return value_option;
            }
        }
    }
}

#[derive(Debug)]
pub struct ThreadData {
    pub stack: SmallVec<[CallFrame; 10]>,
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
