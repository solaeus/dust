use std::fmt::{self, Display, Formatter};

use tracing::{info, trace};

use crate::{
    vm::{FunctionCall, Register},
    Chunk, DustString, Value,
};

use super::{record::Record, CallStack, RunAction};

pub struct Thread {
    chunk: Chunk,
}

impl Thread {
    pub fn new(chunk: Chunk) -> Self {
        Thread { chunk }
    }

    pub fn run(&mut self) -> Option<Value> {
        let mut call_stack = CallStack::with_capacity(self.chunk.prototypes.len() + 1);
        let mut records = Vec::with_capacity(self.chunk.prototypes.len() + 1);

        let main_call = FunctionCall {
            name: self.chunk.name.clone(),
            return_register: 0,
            ip: 0,
        };
        let main_record = Record::new(&self.chunk);

        call_stack.push(main_call);
        records.push(main_record);

        let mut active_record = &mut records[0];

        info!(
            "Starting thread with {}",
            active_record
                .as_function()
                .name
                .unwrap_or_else(|| DustString::from("anonymous"))
        );

        loop {
            trace!(
                "Run \"{}\" | IP = {}",
                active_record
                    .name()
                    .cloned()
                    .unwrap_or_else(|| DustString::from("anonymous")),
                active_record.ip
            );

            let instruction = active_record.chunk.instructions[active_record.ip];
            let action = RunAction::from(instruction);
            let signal = (action.logic)(action.instruction, active_record);

            trace!("Thread Signal: {}", signal);

            active_record.ip += 1;

            match signal {
                ThreadSignal::Continue => {}
                ThreadSignal::Call {
                    function_register,
                    return_register,
                    argument_count,
                } => {
                    let function = active_record
                        .open_register(function_register)
                        .as_function()
                        .unwrap();
                    let first_argument_register = return_register - argument_count;
                    let mut arguments = Vec::with_capacity(argument_count as usize);

                    for register_index in first_argument_register..return_register {
                        let value = active_record.clone_register_value_or_constant(register_index);

                        arguments.push(value);
                    }

                    trace!("Passing arguments: {arguments:?}");

                    let prototype = &self.chunk.prototypes[function.prototype_index as usize];
                    let next_record = Record::new(prototype);
                    let next_call = FunctionCall {
                        name: next_record.name().cloned(),
                        return_register,
                        ip: active_record.ip,
                    };

                    call_stack.push(next_call);
                    records.push(next_record);

                    active_record = records.last_mut().unwrap();

                    for (index, argument) in arguments.into_iter().enumerate() {
                        active_record.set_register(index as u8, Register::Value(argument));
                    }
                }
                ThreadSignal::LoadFunction {
                    destination,
                    prototype_index,
                } => {
                    let function_record_index = prototype_index as usize;
                    let function = self.chunk.prototypes[function_record_index].as_function();
                    let register = Register::Value(Value::Function(function));

                    active_record.set_register(destination, register);
                }
                ThreadSignal::Return {
                    should_return_value,
                    return_register,
                } => {
                    trace!("Returning with call stack:\n{call_stack}");

                    let return_value = if should_return_value {
                        Some(
                            active_record
                                .empty_register_or_clone_constant(return_register, Register::Empty),
                        )
                    } else {
                        None
                    };

                    let current_call = call_stack.pop_or_panic();
                    let _current_record = records.pop().unwrap();
                    let destination = current_call.return_register;

                    if call_stack.is_empty() {
                        return if should_return_value {
                            Some(return_value.unwrap())
                        } else {
                            None
                        };
                    }

                    let outer_record = records.last_mut().unwrap();

                    if should_return_value {
                        outer_record
                            .set_register(destination, Register::Value(return_value.unwrap()));
                    }

                    active_record = outer_record;
                }
            }
        }
    }
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
