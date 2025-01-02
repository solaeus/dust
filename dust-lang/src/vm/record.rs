use std::mem::replace;

use smallvec::SmallVec;
use tracing::trace;

use crate::{DustString, Function, FunctionType, Local, Span, Value};

use super::{run_action::RunAction, Pointer, Register};

#[derive(Debug)]
pub struct Record {
    pub ip: usize,
    pub actions: SmallVec<[RunAction; 32]>,

    stack: Vec<Register>,
    last_assigned_register: Option<u8>,

    name: Option<DustString>,
    r#type: FunctionType,

    positions: SmallVec<[Span; 32]>,
    constants: SmallVec<[Value; 16]>,
    locals: SmallVec<[Local; 8]>,

    index: u8,
}

impl Record {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        actions: SmallVec<[RunAction; 32]>,
        last_assigned_register: Option<u8>,
        name: Option<DustString>,
        r#type: FunctionType,
        positions: SmallVec<[Span; 32]>,
        constants: SmallVec<[Value; 16]>,
        locals: SmallVec<[Local; 8]>,
        stack_size: usize,
        index: u8,
    ) -> Self {
        Self {
            ip: 0,
            actions,
            stack: vec![Register::Empty; stack_size],
            last_assigned_register,
            name,
            r#type,
            positions,
            constants,
            locals,
            index,
        }
    }

    pub fn name(&self) -> Option<&DustString> {
        self.name.as_ref()
    }

    pub fn index(&self) -> u8 {
        self.index
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

    pub fn as_function(&self) -> Function {
        Function {
            name: self.name.clone(),
            r#type: self.r#type.clone(),
            record_index: self.index,
        }
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
        self.last_assigned_register = Some(to_register);
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
        trace!("Open register R{register_index}");

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
        trace!("Open register R{register_index}");

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
