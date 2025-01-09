use std::fmt::{self, Display, Formatter};

use tracing::info;

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
        let mut call_stack = Stack::with_capacity(self.chunk.prototypes.len() + 1);
        let mut records = Stack::with_capacity(self.chunk.prototypes.len() + 1);
        let main_call = FunctionCall {
            name: self.chunk.name.clone(),
            return_register: 0,
            ip: 0,
        };
        let main_record = Record::new(&self.chunk);

        info!(
            "Starting thread with {}",
            self.chunk
                .name
                .clone()
                .unwrap_or_else(|| DustString::from("anonymous"))
        );

        call_stack.push(main_call);
        records.push(main_record);

        let mut thread_data = ThreadData {
            call_stack,
            records,
        };

        let mut next_action = RunAction::from(*self.chunk.instructions.first().unwrap());

        loop {
            let signal = (next_action.logic)(next_action.instruction, &mut thread_data);

            match signal {
                ThreadSignal::Continue(action) => {
                    next_action = action;
                }
                ThreadSignal::End(value_option) => {
                    info!("Thread ended");

                    return value_option;
                }
            }
        }
    }
}

#[derive(Debug)]
pub struct ThreadData<'a> {
    pub call_stack: Stack<FunctionCall>,
    pub records: Stack<Record<'a>>,
}

#[derive(Debug)]
pub enum ThreadSignal {
    Continue(RunAction),
    End(Option<Value>),
}

impl Display for ThreadSignal {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}
