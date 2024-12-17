use std::mem::replace;

use smallvec::SmallVec;

use crate::{Local, Span, Value};

use super::{runner::RunAction, Pointer, Register};

pub struct Record {
    pub ip: usize,
    pub actions: SmallVec<[RunAction; 32]>,
    positions: SmallVec<[Span; 32]>,

    stack: Vec<Register>,
    constants: SmallVec<[Value; 16]>,
    locals: SmallVec<[Local; 8]>,

    last_assigned_register: Option<u8>,
}

impl Record {
    pub fn new(
        stack: Vec<Register>,
        constants: SmallVec<[Value; 16]>,
        locals: SmallVec<[Local; 8]>,
        actions: SmallVec<[RunAction; 32]>,
        positions: SmallVec<[Span; 32]>,
    ) -> Self {
        Self {
            ip: 0,
            actions,
            positions,
            stack,
            constants,
            locals,
            last_assigned_register: None,
        }
    }

    pub fn stack_size(&self) -> usize {
        self.stack.len()
    }

    pub fn current_position(&self) -> Span {
        self.positions[self.ip]
    }

    pub fn last_assigned_register(&self) -> Option<u8> {
        self.last_assigned_register
    }

    pub(crate) fn follow_pointer(&self, pointer: Pointer) -> &Value {
        log::trace!("Follow pointer {pointer}");

        match pointer {
            Pointer::Stack(register_index) => self.open_register(register_index),
            Pointer::Constant(constant_index) => self.get_constant(constant_index),
        }
    }

    pub(crate) fn follow_pointer_allow_empty(&self, pointer: Pointer) -> Option<&Value> {
        log::trace!("Follow pointer {pointer}");

        match pointer {
            Pointer::Stack(register_index) => self.open_register_allow_empty(register_index),
            Pointer::Constant(constant_index) => {
                let constant = self.get_constant(constant_index);

                Some(constant)
            }
        }
    }

    pub fn get_register(&self, register_index: u8) -> &Register {
        log::trace!("Get register R{register_index}");

        let register_index = register_index as usize;

        assert!(
            register_index < self.stack.len(),
            "VM Error: Register index out of bounds"
        );

        &self.stack[register_index]
    }

    pub fn set_register(&mut self, to_register: u8, register: Register) {
        self.last_assigned_register = Some(to_register);
        let to_register = to_register as usize;

        assert!(
            to_register < self.stack.len(),
            "VM Error: Register index out of bounds"
        );

        self.stack[to_register] = register;
    }

    pub fn open_register(&self, register_index: u8) -> &Value {
        log::trace!("Open register R{register_index}");

        let register_index = register_index as usize;

        assert!(
            register_index < self.stack.len(),
            "VM Error: Register index out of bounds"
        );

        let register = &self.stack[register_index];

        match register {
            Register::Value(value) => value,
            Register::Pointer(pointer) => self.follow_pointer(*pointer),
            Register::Empty => panic!("VM Error: Register {register_index} is empty"),
        }
    }

    pub fn open_register_allow_empty(&self, register_index: u8) -> Option<&Value> {
        log::trace!("Open register R{register_index}");

        let register_index = register_index as usize;

        assert!(
            register_index < self.stack.len(),
            "VM Error: Register index out of bounds"
        );

        let register = &self.stack[register_index];

        match register {
            Register::Value(value) => Some(value),
            Register::Pointer(pointer) => Some(self.follow_pointer(*pointer)),
            Register::Empty => None,
        }
    }

    pub fn replace_register_or_clone_constant(
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

    /// DRY helper to get a value from an Argument
    pub fn get_argument(&self, index: u8, is_constant: bool) -> &Value {
        if is_constant {
            self.get_constant(index)
        } else {
            self.open_register(index)
        }
    }

    pub fn get_constant(&self, constant_index: u8) -> &Value {
        let constant_index = constant_index as usize;

        assert!(
            constant_index < self.constants.len(),
            "VM Error: Constant index out of bounds"
        );

        &self.constants[constant_index]
    }

    pub fn get_local_register(&self, local_index: u8) -> u8 {
        let local_index = local_index as usize;

        assert!(
            local_index < self.locals.len(),
            "VM Error: Local index out of bounds"
        );

        self.locals[local_index].register_index
    }
}
