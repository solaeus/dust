use std::mem::replace;

use tracing::{info, trace};

use crate::{vm::FunctionCall, Argument, Chunk, DustString, Span, Value};

use super::{Pointer, Register, RunAction, Stack};

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
        let main_call = FunctionCall::new(&self.chunk, 0);

        call_stack.push(main_call);

        let first_action = RunAction::from(*self.chunk.instructions.first().unwrap());
        let mut thread_data = ThreadData {
            call_stack,
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
                    let value =
                        thread_data.empty_register_or_clone_constant_unchecked(register_index);

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
    pub call_stack: Stack<FunctionCall<'a>>,
    pub next_action: RunAction,
    pub return_value_index: Option<u8>,
}

impl ThreadData<'_> {
    pub fn current_position(&self) -> Span {
        let current_call = self.call_stack.last_unchecked();

        current_call.chunk.positions[current_call.ip]
    }

    pub(crate) fn follow_pointer_unchecked(&self, pointer: Pointer) -> &Value {
        trace!("Follow pointer {pointer}");

        match pointer {
            Pointer::Register(register_index) => self.open_register_unchecked(register_index),
            Pointer::Constant(constant_index) => self.get_constant_unchecked(constant_index),
            Pointer::Stack(stack_index, register_index) => unsafe {
                let register = self
                    .call_stack
                    .get_unchecked(stack_index)
                    .registers
                    .get_unchecked(register_index as usize);

                match register {
                    Register::Value(value) => value,
                    Register::Pointer(pointer) => self.follow_pointer_unchecked(*pointer),
                    Register::Empty => panic!("VM Error: Register {register_index} is empty"),
                }
            },
        }
    }

    pub fn get_register_unchecked(&self, register_index: u8) -> &Register {
        trace!("Get register R{register_index}");

        let register_index = register_index as usize;

        if cfg!(debug_assertions) {
            &self.call_stack.last_unchecked().registers[register_index]
        } else {
            unsafe {
                self.call_stack
                    .last_unchecked()
                    .registers
                    .get_unchecked(register_index)
            }
        }
    }

    pub fn set_register(&mut self, to_register: u8, register: Register) {
        let to_register = to_register as usize;

        self.call_stack.last_mut_unchecked().registers[to_register] = register;
    }

    pub fn open_register_unchecked(&self, register_index: u8) -> &Value {
        let register_index = register_index as usize;

        let register = if cfg!(debug_assertions) {
            &self.call_stack.last_unchecked().registers[register_index]
        } else {
            unsafe {
                self.call_stack
                    .last_unchecked()
                    .registers
                    .get_unchecked(register_index)
            }
        };

        match register {
            Register::Value(value) => {
                trace!("Register R{register_index} opened to value {value}");

                value
            }
            Register::Pointer(pointer) => {
                trace!("Open register R{register_index} opened to pointer {pointer}");

                self.follow_pointer_unchecked(*pointer)
            }
            Register::Empty => panic!("VM Error: Register {register_index} is empty"),
        }
    }

    pub fn open_register_allow_empty_unchecked(&self, register_index: u8) -> Option<&Value> {
        trace!("Open register R{register_index}");

        let register = self.get_register_unchecked(register_index);

        match register {
            Register::Value(value) => {
                trace!("Register R{register_index} openned to value {value}");

                Some(value)
            }
            Register::Pointer(pointer) => {
                trace!("Open register R{register_index} openned to pointer {pointer}");

                Some(self.follow_pointer_unchecked(*pointer))
            }
            Register::Empty => None,
        }
    }

    pub fn empty_register_or_clone_constant_unchecked(&mut self, register_index: u8) -> Value {
        let register_index = register_index as usize;
        let old_register = replace(
            &mut self.call_stack.last_mut_unchecked().registers[register_index],
            Register::Empty,
        );

        match old_register {
            Register::Value(value) => value,
            Register::Pointer(pointer) => match pointer {
                Pointer::Register(register_index) => {
                    self.empty_register_or_clone_constant_unchecked(register_index)
                }
                Pointer::Constant(constant_index) => {
                    self.get_constant_unchecked(constant_index).clone()
                }
                Pointer::Stack(stack_index, register_index) => {
                    let call = self.call_stack.get_unchecked_mut(stack_index);

                    let old_register = replace(
                        &mut call.registers[register_index as usize],
                        Register::Empty,
                    );

                    match old_register {
                        Register::Value(value) => value,
                        Register::Pointer(pointer) => {
                            self.follow_pointer_unchecked(pointer).clone()
                        }
                        Register::Empty => panic!("VM Error: Register {register_index} is empty"),
                    }
                }
            },
            Register::Empty => panic!("VM Error: Register {register_index} is empty"),
        }
    }

    pub fn clone_register_value_or_constant_unchecked(&self, register_index: u8) -> Value {
        let register = self.get_register_unchecked(register_index);

        match register {
            Register::Value(value) => value.clone(),
            Register::Pointer(pointer) => match pointer {
                Pointer::Register(register_index) => {
                    self.open_register_unchecked(*register_index).clone()
                }
                Pointer::Constant(constant_index) => {
                    self.get_constant_unchecked(*constant_index).clone()
                }
                Pointer::Stack(stack_index, register_index) => {
                    let call = self.call_stack.get_unchecked(*stack_index);
                    let register = &call.registers[*register_index as usize];

                    match register {
                        Register::Value(value) => value.clone(),
                        Register::Pointer(pointer) => {
                            self.follow_pointer_unchecked(*pointer).clone()
                        }
                        Register::Empty => panic!("VM Error: Register {register_index} is empty"),
                    }
                }
            },
            Register::Empty => panic!("VM Error: Register {register_index} is empty"),
        }
    }

    /// DRY helper to get a value from an Argument
    pub fn get_argument_unchecked(&self, argument: Argument) -> &Value {
        match argument {
            Argument::Constant(constant_index) => self.get_constant_unchecked(constant_index),
            Argument::Register(register_index) => self.open_register_unchecked(register_index),
        }
    }

    pub fn get_constant_unchecked(&self, constant_index: u8) -> &Value {
        let constant_index = constant_index as usize;

        if cfg!(debug_assertions) {
            &self.call_stack.last().unwrap().chunk.constants[constant_index]
        } else {
            unsafe {
                self.call_stack
                    .last_unchecked()
                    .chunk
                    .constants
                    .get_unchecked(constant_index)
            }
        }
    }

    pub fn get_local_register(&self, local_index: u8) -> u8 {
        let local_index = local_index as usize;
        let chunk = self.call_stack.last_unchecked().chunk;

        assert!(
            local_index < chunk.locals.len(),
            "VM Error: Local index out of bounds"
        );

        chunk.locals[local_index].register_index
    }
}
