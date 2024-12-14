//! Virtual machine and errors
mod runner;

use std::fmt::{self, Display, Formatter};

use runner::Runner;

use crate::{compile, instruction::*, Chunk, DustError, Span, Value};

pub fn run(source: &str) -> Result<Option<Value>, DustError> {
    let chunk = compile(source)?;
    let vm = Vm::new(&chunk, None, None);

    Ok(vm.run())
}

/// Dust virtual machine.
///
/// See the [module-level documentation](index.html) for more information.
#[derive(Debug)]
pub struct Vm<'a> {
    stack: Vec<Register>,

    runners: Vec<Runner>,
    chunk: &'a Chunk,
    parent: Option<&'a Vm<'a>>,

    ip: usize,
    last_assigned_register: Option<u8>,
    return_value: Option<Value>,
}

impl<'a> Vm<'a> {
    pub fn new(chunk: &'a Chunk, parent: Option<&'a Vm<'a>>, runners: Option<Vec<Runner>>) -> Self {
        let stack = vec![Register::Empty; chunk.stack_size()];
        let runners = runners.unwrap_or_else(|| {
            let mut runners = Vec::with_capacity(chunk.instructions().len());

            for (instruction, _) in chunk.instructions() {
                let runner = Runner::new(*instruction);

                runners.push(runner);
            }

            runners
        });

        Self {
            chunk,
            runners,
            stack,
            parent,
            ip: 0,
            last_assigned_register: None,
            return_value: None,
        }
    }

    pub fn chunk(&'a self) -> &'a Chunk {
        self.chunk
    }

    pub fn current_position(&self) -> Span {
        let index = self.ip.saturating_sub(1);
        let (_, position) = self.chunk.instructions()[index];

        position
    }

    pub fn run(mut self) -> Option<Value> {
        self.execute_next_runner();

        self.return_value
    }

    pub fn execute_next_runner(&mut self) {
        assert!(
            self.ip < self.runners.len(),
            "Runtime Error: IP out of bounds"
        );

        let runner = self.runners[self.ip];

        runner.run(self);
    }

    pub(crate) fn follow_pointer(&self, pointer: Pointer) -> &Value {
        log::trace!("Follow pointer {pointer}");

        match pointer {
            Pointer::Stack(register_index) => self.open_register(register_index),
            Pointer::Constant(constant_index) => self.get_constant(constant_index),
            Pointer::ParentStack(register_index) => {
                assert!(self.parent.is_some(), "Vm Error: Expected parent");

                self.parent.unwrap().open_register(register_index)
            }
            Pointer::ParentConstant(constant_index) => {
                assert!(self.parent.is_some(), "Vm Error: Expected parent");

                self.parent.unwrap().get_constant(constant_index)
            }
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
            Pointer::ParentStack(register_index) => {
                assert!(self.parent.is_some(), "Vm Error: Expected parent");

                self.parent
                    .unwrap()
                    .open_register_allow_empty(register_index)
            }
            Pointer::ParentConstant(constant_index) => {
                assert!(self.parent.is_some(), "Vm Error: Expected parent");

                let constant = self.parent.unwrap().get_constant(constant_index);

                Some(constant)
            }
        }
    }

    fn open_register(&self, register_index: u8) -> &Value {
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

    fn open_register_allow_empty(&self, register_index: u8) -> Option<&Value> {
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

    /// DRY helper to get a value from an Argument
    fn get_argument(&self, index: u8, is_constant: bool) -> &Value {
        if is_constant {
            self.get_constant(index)
        } else {
            self.open_register(index)
        }
    }

    fn set_register(&mut self, to_register: u8, register: Register) {
        self.last_assigned_register = Some(to_register);
        let to_register = to_register as usize;

        assert!(
            to_register < self.stack.len(),
            "VM Error: Register index out of bounds"
        );

        self.stack[to_register] = register;
    }

    fn get_constant(&self, constant_index: u8) -> &Value {
        let constant_index = constant_index as usize;
        let constants = self.chunk.constants();

        assert!(
            constant_index < constants.len(),
            "VM Error: Constant index out of bounds"
        );

        &constants[constant_index]
    }

    fn get_local_register(&self, local_index: u8) -> u8 {
        let local_index = local_index as usize;
        let locals = self.chunk.locals();

        assert!(
            local_index < locals.len(),
            "VM Error: Local index out of bounds"
        );

        locals[local_index].register_index
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum Register {
    Empty,
    Value(Value),
    Pointer(Pointer),
}

impl Display for Register {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Self::Empty => write!(f, "empty"),
            Self::Value(value) => write!(f, "{}", value),
            Self::Pointer(pointer) => write!(f, "{}", pointer),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, PartialOrd, Ord)]
pub enum Pointer {
    Stack(u8),
    Constant(u8),
    ParentStack(u8),
    ParentConstant(u8),
}

impl Display for Pointer {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Self::Stack(index) => write!(f, "R{}", index),
            Self::Constant(index) => write!(f, "C{}", index),
            Self::ParentStack(index) => write!(f, "PR{}", index),
            Self::ParentConstant(index) => write!(f, "PC{}", index),
        }
    }
}
