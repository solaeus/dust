use std::mem::replace;

use tracing::trace;

use crate::{Argument, Chunk, DustString, Function, Span, Value};

use super::{Pointer, Register};

#[derive(Debug)]
pub struct Record<'a> {
    pub ip: usize,
    pub chunk: &'a Chunk,
    stack: Vec<Register>,
}

impl<'a> Record<'a> {
    #[allow(clippy::too_many_arguments)]
    pub fn new(chunk: &'a Chunk) -> Self {
        Self {
            ip: 0,
            stack: vec![Register::Empty; chunk.stack_size],
            chunk,
        }
    }

    pub fn name(&self) -> Option<&DustString> {
        self.chunk.name.as_ref()
    }

    pub fn stack_size(&self) -> usize {
        self.stack.len()
    }

    pub fn current_position(&self) -> Span {
        self.chunk.positions[self.ip]
    }

    pub fn as_function(&self) -> Function {
        self.chunk.as_function()
    }

    pub(crate) fn follow_pointer(&self, pointer: Pointer) -> &Value {
        trace!("Follow pointer {pointer}");

        match pointer {
            Pointer::Stack(register_index) => self.open_register(register_index),
            Pointer::Constant(constant_index) => self.get_constant(constant_index),
        }
    }

    pub fn get_register(&self, register_index: u8) -> &Register {
        trace!("Get register R{register_index}");

        let register_index = register_index as usize;

        assert!(
            register_index < self.stack.len(),
            "VM Error: Register index out of bounds"
        );

        &self.stack[register_index]
    }

    pub fn set_register(&mut self, to_register: u8, register: Register) {
        let to_register = to_register as usize;

        assert!(
            to_register < self.stack.len(),
            "VM Error: Register index out of bounds"
        );

        self.stack[to_register] = register;
    }

    pub fn reserve_registers(&mut self, count: usize) {
        for _ in 0..count {
            self.stack.push(Register::Empty);
        }
    }

    pub fn open_register(&self, register_index: u8) -> &Value {
        let register_index = register_index as usize;

        assert!(
            register_index < self.stack.len(),
            "VM Error: Register index out of bounds"
        );

        let register = &self.stack[register_index];

        match register {
            Register::Value(value) => {
                trace!("Register R{register_index} openned to value {value}");

                value
            }
            Register::Pointer(pointer) => {
                trace!("Open register R{register_index} openned to pointer {pointer}");

                self.follow_pointer(*pointer)
            }
            Register::Empty => panic!("VM Error: Register {register_index} is empty"),
        }
    }

    pub fn open_register_allow_empty(&self, register_index: u8) -> Option<&Value> {
        trace!("Open register R{register_index}");

        let register_index = register_index as usize;

        assert!(
            register_index < self.stack.len(),
            "VM Error: Register index out of bounds"
        );

        let register = &self.stack[register_index];

        match register {
            Register::Value(value) => {
                trace!("Register R{register_index} openned to value {value}");

                Some(value)
            }
            Register::Pointer(pointer) => {
                trace!("Open register R{register_index} openned to pointer {pointer}");

                Some(self.follow_pointer(*pointer))
            }
            Register::Empty => None,
        }
    }

    pub fn empty_register_or_clone_constant(
        &mut self,
        register_index: u8,
        new_register: Register,
    ) -> Value {
        let register_index = register_index as usize;

        assert!(
            register_index < self.stack.len(),
            "VM Error: Register index out of bounds"
        );

        let old_register = replace(&mut self.stack[register_index], new_register);

        match old_register {
            Register::Value(value) => value,
            Register::Pointer(pointer) => match pointer {
                Pointer::Stack(register_index) => self.open_register(register_index).clone(),
                Pointer::Constant(constant_index) => self.get_constant(constant_index).clone(),
            },
            Register::Empty => panic!("VM Error: Register {register_index} is empty"),
        }
    }

    pub fn clone_register_value_or_constant(&self, register_index: u8) -> Value {
        assert!(
            (register_index as usize) < self.stack.len(),
            "VM Error: Register index out of bounds"
        );

        let register = &self.stack[register_index as usize];

        match register {
            Register::Value(value) => value.clone(),
            Register::Pointer(pointer) => match pointer {
                Pointer::Stack(register_index) => self.open_register(*register_index).clone(),
                Pointer::Constant(constant_index) => self.get_constant(*constant_index).clone(),
            },
            Register::Empty => panic!("VM Error: Register {register_index} is empty"),
        }
    }

    /// DRY helper to get a value from an Argument
    pub fn get_argument(&self, argument: Argument) -> &Value {
        match argument {
            Argument::Constant(constant_index) => self.get_constant(constant_index),
            Argument::Register(register_index) => self.open_register(register_index),
        }
    }

    pub fn get_constant(&self, constant_index: u8) -> &Value {
        let constant_index = constant_index as usize;

        assert!(
            constant_index < self.chunk.constants.len(),
            "VM Error: Constant index out of bounds"
        );

        &self.chunk.constants[constant_index]
    }

    pub fn get_local_register(&self, local_index: u8) -> u8 {
        let local_index = local_index as usize;

        assert!(
            local_index < self.chunk.locals.len(),
            "VM Error: Local index out of bounds"
        );

        self.chunk.locals[local_index].register_index
    }
}
