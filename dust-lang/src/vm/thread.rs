use tracing::{info, trace};

use crate::{
    vm::{FunctionCall, Register},
    Chunk, DustString, Value,
};

use super::{record::Record, CallStack};

pub struct Thread {
    call_stack: CallStack,
    records: Vec<Record>,
}

impl Thread {
    pub fn new(chunk: Chunk) -> Self {
        let call_stack = CallStack::with_capacity(chunk.prototypes().len() + 1);
        let mut records = Vec::with_capacity(chunk.prototypes().len() + 1);

        chunk.into_records(&mut records);

        Thread {
            call_stack,
            records,
        }
    }

    pub fn run(&mut self) -> Option<Value> {
        let mut active = &mut self.records[0];
        let mut current_call = FunctionCall {
            name: active.name().cloned(),
            record_index: active.index(),
            return_register: active.stack_size() as u8 - 1,
            ip: active.ip,
        };

        info!(
            "Starting thread with {}",
            active
                .as_function()
                .name
                .unwrap_or_else(|| DustString::from("anonymous"))
        );

        loop {
            trace!(
                "Run \"{}\" | Record = {} | IP = {}",
                active
                    .name()
                    .cloned()
                    .unwrap_or_else(|| DustString::from("anonymous")),
                active.index(),
                active.ip
            );

            if active.ip >= active.actions.len() {
                return None;
            }

            let action = active.actions[active.ip];
            let signal = (action.logic)(action.data, active);

            trace!("Thread Signal: {:?}", signal);

            active.ip += 1;

            match signal {
                ThreadSignal::Continue => {}
                ThreadSignal::Call {
                    record_index,
                    return_register,
                    argument_count,
                } => {
                    let record_index = record_index as usize;
                    let first_argument_register = return_register - argument_count;
                    let mut arguments = Vec::with_capacity(argument_count as usize);

                    for register_index in first_argument_register..return_register {
                        let value = active.clone_register_value_or_constant(register_index);

                        arguments.push(value);
                    }

                    if record_index == active.index() as usize {
                        trace!("Recursion detected");

                        if let Some(record) = self.call_stack.last_mut() {
                            record.ip = active.ip;
                        }

                        active.ip = 0;
                    }

                    let next_call = FunctionCall {
                        name: active.name().cloned(),
                        record_index: active.index(),
                        return_register,
                        ip: active.ip,
                    };

                    if self
                        .call_stack
                        .last()
                        .is_some_and(|call| call != &next_call)
                        || self.call_stack.is_empty()
                    {
                        self.call_stack.push(current_call);
                    }

                    current_call = next_call;

                    active = &mut self.records[record_index];

                    for (index, argument) in arguments.into_iter().enumerate() {
                        active.set_register(index as u8, Register::Value(argument));
                    }
                }
                ThreadSignal::LoadFunction {
                    from_record_index,
                    to_register_index,
                } => {
                    let function_record_index = from_record_index as usize;
                    let original_record_index = active.index() as usize;

                    active = &mut self.records[function_record_index];

                    let function = active.as_function();
                    let register = Register::Value(Value::Function(function));

                    active = &mut self.records[original_record_index];

                    active.set_register(to_register_index, register);
                }
                ThreadSignal::Return {
                    should_return_value,
                    return_register,
                } => {
                    trace!("\n{:#?}{}", self.call_stack, current_call);

                    let outer_call = if let Some(call) = self.call_stack.pop() {
                        call
                    } else if should_return_value {
                        let return_value = active
                            .empty_register_or_clone_constant(return_register, Register::Empty);
                        let next_call = self.call_stack.pop();

                        if next_call.is_none() {
                            return Some(return_value);
                        }

                        let next_index = active.index() as usize - 1;
                        let next_record = &mut self.records[next_index];

                        next_record.set_register(
                            current_call.return_register,
                            Register::Value(return_value),
                        );

                        current_call = next_call.unwrap();
                        active = next_record;

                        continue;
                    } else {
                        return None;
                    };
                    let record_index = outer_call.record_index as usize;
                    let destination = current_call.return_register;

                    if should_return_value {
                        let return_value = active
                            .empty_register_or_clone_constant(return_register, Register::Empty);
                        let outer_record = &mut self.records[record_index];

                        outer_record.set_register(destination, Register::Value(return_value));
                    }

                    active = &mut self.records[record_index];
                    current_call = outer_call;
                }
            }
        }
    }
}

#[derive(Debug)]
pub enum ThreadSignal {
    Continue,
    Call {
        record_index: u8,
        return_register: u8,
        argument_count: u8,
    },
    Return {
        should_return_value: bool,
        return_register: u8,
    },
    LoadFunction {
        from_record_index: u8,
        to_register_index: u8,
    },
}
