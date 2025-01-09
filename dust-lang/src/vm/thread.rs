use tracing::{info, trace};

use crate::{vm::FunctionCall, Chunk, DustString, Value};

use super::{record::Record, RunAction, Stack};

pub struct Thread {
    chunk: Chunk,
}

impl Thread {
    pub fn new(chunk: Chunk) -> Self {
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

        let mut call_stack = Stack::with_capacity(self.chunk.prototypes.len() + 1);
        let mut records = Stack::with_capacity(self.chunk.prototypes.len() + 1);
        let main_call = FunctionCall {
            name: self.chunk.name.clone(),
            return_register: 0, // Never used, the main function's return is the thread's return
            ip: 0,
        };
        let main_record = Record::new(&self.chunk);

        call_stack.push(main_call);
        records.push(main_record);

        let first_action = RunAction::from(*self.chunk.instructions.first().unwrap());
        let mut thread_data = ThreadData {
            call_stack,
            records,
            next_action: first_action,
            return_value_index: None,
        };

        loop {
            trace!("Instruction: {}", thread_data.next_action.instruction);

            let should_end = (thread_data.next_action.logic)(
                thread_data.next_action.instruction,
                &mut thread_data,
            );

            if should_end {
                let return_value = if let Some(register_index) = thread_data.return_value_index {
                    let value = thread_data
                        .records
                        .last_mut_unchecked()
                        .empty_register_or_clone_constant_unchecked(register_index);

                    Some(value)
                } else {
                    None
                };

                return return_value;
            }
        }
    }
}

#[derive(Debug)]
pub struct ThreadData<'a> {
    pub call_stack: Stack<FunctionCall>,
    pub records: Stack<Record<'a>>,
    pub next_action: RunAction,
    pub return_value_index: Option<u8>,
}
