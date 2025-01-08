use std::fmt::{self, Display, Formatter};

use tracing::{info, trace};

use crate::{vm::FunctionCall, Chunk, DustString, Value};

use super::{record::Record, RecordAction, Stack};

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

        loop {
            let active_record = thread_data.records.last_unchecked();
            let instruction = active_record.chunk.instructions[active_record.ip];
            let record_action = RecordAction::from(instruction);
            let value_option = (record_action.logic)(record_action.instruction, &mut thread_data);

            if thread_data.call_stack.is_empty() {
                trace!("Returning {value_option:?} from function");

                return value_option;
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
    Continue,
    Call {
        function_register: u8,
        return_register: u8,
        argument_count: u8,
    },
    Return {
        should_return_value: bool,
        return_register: u8,
    },
    LoadFunction {
        prototype_index: u8,
        destination: u8,
    },
}

impl Display for ThreadSignal {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}
