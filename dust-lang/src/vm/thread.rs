use crate::{Chunk, Value};

use super::{call_stack::CallStack, record::Record, runner::RunAction, FunctionCall, VmError};

fn create_records(chunk: Chunk, records: &mut Vec<Record>) {
    let (_, _, instructions, positions, constants, locals, prototypes, stack_size) =
        chunk.take_data();
    let actions = instructions
        .into_iter()
        .map(|instruction| RunAction::from(instruction))
        .collect();
    let record = Record::new(
        Vec::with_capacity(stack_size),
        constants,
        locals,
        actions,
        positions,
    );

    for chunk in prototypes {
        create_records(chunk, records);
    }

    records.push(record);
}

pub struct Thread {
    call_stack: CallStack,
    records: Vec<Record>,
    return_register: Option<u8>,
}

impl Thread {
    pub fn new(chunk: Chunk) -> Self {
        let call_stack = CallStack::new();
        let mut records = Vec::with_capacity(chunk.prototypes().len());

        create_records(chunk, &mut records);

        Thread {
            call_stack,
            records,
            return_register: None,
        }
    }

    pub fn run(&mut self) -> Option<Value> {
        assert!(!self.call_stack.is_empty());

        let mut record = &mut self.records[0];

        loop {
            assert!(
                record.ip < record.actions.len(),
                "{}",
                VmError::InstructionIndexOutOfBounds {
                    call_stack: self.call_stack.clone(),
                    ip: record.ip,
                }
            );

            let action = record.actions[record.ip];
            let signal = (action.logic)(action.data, &mut record);

            match signal {
                ThreadSignal::Continue => {
                    record.ip += 1;
                }
                ThreadSignal::Call(FunctionCall {
                    record_index,
                    return_register,
                    ..
                }) => {
                    record = &mut self.records[record_index];
                    self.return_register = Some(return_register);
                }
                ThreadSignal::Return(value_option) => {
                    let outer_call = self.call_stack.pop();

                    if outer_call.is_none() {
                        return value_option;
                    }
                }
            }
        }
    }
}

pub enum ThreadSignal {
    Continue,
    Call(FunctionCall),
    Return(Option<Value>),
}
