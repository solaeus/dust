use std::{mem::replace, sync::Arc, thread::JoinHandle};

use tracing::{info, trace};

use crate::{
    Chunk, DustString, Operand, Span, Value,
    vm::{CallFrame, action::ActionSequence},
};

use super::{Pointer, Register};

pub struct Thread {
    chunk: Arc<Chunk>,
    call_stack: Vec<CallFrame>,
    return_value_index: Option<Option<usize>>,
    spawned_threads: Vec<JoinHandle<()>>,
}

impl Thread {
    pub fn new(chunk: Arc<Chunk>) -> Self {
        let mut call_stack = Vec::with_capacity(chunk.prototypes.len() + 1);
        let main_call = CallFrame::new(Arc::clone(&chunk), 0);

        call_stack.push(main_call);

        Thread {
            chunk,
            call_stack,
            return_value_index: None,
            spawned_threads: Vec::new(),
        }
    }

    pub fn run(mut self) -> Option<Value> {
        info!(
            "Starting thread with {}",
            self.chunk
                .name
                .clone()
                .unwrap_or_else(|| DustString::from("anonymous"))
        );

        let main_call = self.current_frame();
        let action_sequence = ActionSequence::new(&main_call.chunk.instructions);

        loop {
            let current_frame = self.current_frame_mut();
            let ip = {
                let ip = current_frame.ip;
                current_frame.ip += 1;

                ip
            };
            let current_action = if cfg!(debug_assertions) {
                action_sequence.actions.get(ip).unwrap()
            } else {
                unsafe { action_sequence.actions.get_unchecked(ip) }
            };

            trace!("Operation: {}", current_action.instruction.operation);

            (current_action.logic)(current_action.instruction, &mut self);

            if let Some(return_index_option) = self.return_value_index {
                if let Some(return_index) = return_index_option {
                    let return_value = self.open_register_unchecked(return_index as u16).clone();

                    return Some(return_value);
                } else {
                    return None;
                }
            }
        }
    }

    pub fn current_position(&self) -> Span {
        let current_frame = self.current_frame();

        current_frame.chunk.positions[current_frame.ip]
    }

    pub fn current_frame(&self) -> &CallFrame {
        if cfg!(debug_assertions) {
            self.call_stack.last().unwrap()
        } else {
            unsafe { self.call_stack.last().unwrap_unchecked() }
        }
    }

    pub fn current_frame_mut(&mut self) -> &mut CallFrame {
        if cfg!(debug_assertions) {
            self.call_stack.last_mut().unwrap()
        } else {
            unsafe { self.call_stack.last_mut().unwrap_unchecked() }
        }
    }

    pub(crate) fn follow_pointer_unchecked(&self, pointer: Pointer) -> &Value {
        trace!("Follow {pointer}");

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

    pub fn get_register_unchecked(&self, register_index: u16) -> &Register {
        trace!("Get R{register_index}");

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

    pub fn set_register(&mut self, to_register: u16, register: Register) {
        let to_register = to_register as usize;

        self.call_stack.last_mut_unchecked().registers[to_register] = register;
    }

    pub fn open_register_unchecked(&self, register_index: u16) -> &Value {
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

        trace!("Open R{register_index} to {register}");

        match register {
            Register::Value(value) => value,
            Register::Pointer(pointer) => self.follow_pointer_unchecked(*pointer),
            Register::Empty => panic!("VM Error: Register {register_index} is empty"),
        }
    }

    pub fn open_register_allow_empty_unchecked(&self, register_index: u16) -> Option<&Value> {
        trace!("Open R{register_index}");

        let register = self.get_register_unchecked(register_index);

        trace!("Open R{register_index} to {register}");

        match register {
            Register::Value(value) => Some(value),
            Register::Pointer(pointer) => Some(self.follow_pointer_unchecked(*pointer)),
            Register::Empty => None,
        }
    }

    pub fn empty_register_or_clone_constant_unchecked(&mut self, register_index: u16) -> Value {
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

    pub fn clone_register_value_or_constant_unchecked(&self, register_index: u16) -> Value {
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
    pub fn get_argument_unchecked(&self, argument: Operand) -> &Value {
        match argument {
            Operand::Constant(constant_index) => self.get_constant_unchecked(constant_index),
            Operand::Register(register_index) => self.open_register_unchecked(register_index),
        }
    }

    pub fn get_constant_unchecked(&self, constant_index: u16) -> &Value {
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

    pub fn get_local_register(&self, local_index: u16) -> u16 {
        let local_index = local_index as usize;
        let chunk = &self.call_stack.last_unchecked().chunk;

        assert!(
            local_index < chunk.locals.len(),
            "VM Error: Local index out of bounds"
        );

        chunk.locals[local_index].register_index
    }
}
