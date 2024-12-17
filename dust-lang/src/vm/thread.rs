use std::mem::swap;

use crate::{vm::Register, Chunk, Value};

use super::{record::Record, runner::RunAction, CallStack, FunctionCall, VmError};

fn create_records(chunk: Chunk, records: &mut Vec<Record>) {
    let (_, _, instructions, positions, constants, locals, prototypes, stack_size) =
        chunk.take_data();
    let actions = instructions.into_iter().map(RunAction::from).collect();
    let record = Record::new(
        vec![Register::Empty; stack_size],
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
}

impl Thread {
    pub fn new(chunk: Chunk) -> Self {
        let call_stack = CallStack::with_capacity(chunk.prototypes().len() + 1);
        let mut records = Vec::with_capacity(chunk.prototypes().len() + 1);

        create_records(chunk, &mut records);

        Thread {
            call_stack,
            records,
        }
    }

    pub fn run(&mut self) -> Option<Value> {
        let (record, remaining_records) = self.records.split_first_mut().unwrap();

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
            let signal = (action.logic)(action.data, record);

            match signal {
                ThreadSignal::Continue => {
                    record.ip += 1;
                }
                ThreadSignal::Call(function_call) => {
                    swap(record, &mut remaining_records[function_call.record_index]);
                    self.call_stack.push(function_call);
                }
                ThreadSignal::Return(should_return_value) => {
                    let returning_call = match self.call_stack.pop() {
                        Some(function_call) => function_call,
                        None => {
                            if should_return_value {
                                return record.last_assigned_register().map(|register| {
                                    record.replace_register_or_clone_constant(
                                        register,
                                        Register::Empty,
                                    )
                                });
                            } else {
                                return None;
                            }
                        }
                    };
                    let outer_call = self.call_stack.last_or_panic();

                    if should_return_value {
                        let return_register = record
                            .last_assigned_register()
                            .unwrap_or_else(|| panic!("Expected return value"));
                        let value = record
                            .replace_register_or_clone_constant(return_register, Register::Empty);

                        swap(record, &mut remaining_records[outer_call.record_index]);

                        record.set_register(returning_call.return_register, Register::Value(value));
                    } else {
                        swap(record, &mut remaining_records[outer_call.record_index]);
                    }
                }
            }
        }
    }
}

pub enum ThreadSignal {
    Continue,
    Call(FunctionCall),
    Return(bool),
}
