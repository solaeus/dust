//! Virtual machine and errors
mod runners;

use std::{
    fmt::{self, Display, Formatter},
    io, iter,
    rc::Rc,
};

use runners::Runner;
use smallvec::SmallVec;

use crate::{
    compile, instruction::*, AbstractValue, AnnotatedError, Chunk, ConcreteValue, DustError,
    NativeFunctionError, Span, Value, ValueError,
};

pub fn run(source: &str) -> Result<Option<Value>, DustError> {
    let chunk = compile(source)?;
    let vm = Vm::new(source, &chunk, None, None);

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
    source: &'a str,
    return_value: Option<Value>,
}

impl<'a> Vm<'a> {
    pub fn new(
        source: &'a str,
        chunk: &'a Chunk,
        parent: Option<&'a Vm<'a>>,
        runners: Option<Vec<Runner>>,
    ) -> Self {
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
            source,
            return_value: None,
        }
    }

    pub fn chunk(&self) -> &Chunk {
        self.chunk
    }

    pub fn source(&self) -> &'a str {
        self.source
    }

    pub fn current_position(&self) -> Span {
        let index = self.ip.saturating_sub(1);
        let (_, position) = self.chunk.instructions()[index];

        position
    }

    pub fn run(mut self) -> Option<Value> {
        while self.ip < self.runners.len() && self.return_value.is_none() {
            self.execute_next_runner();
        }

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

    /// DRY helper for handling JUMP instructions
    fn jump_instructions(&mut self, offset: usize, is_positive: bool) {
        log::trace!(
            "Jumping {}",
            if is_positive {
                format!("+{}", offset)
            } else {
                format!("-{}", offset)
            }
        );

        if is_positive {
            self.ip += offset
        } else {
            self.ip -= offset + 1
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

#[derive(Clone, Debug, PartialEq)]
pub enum VmError {
    // Stack errors
    StackOverflow {
        position: Span,
    },
    StackUnderflow {
        position: Span,
    },

    // Register errors
    EmptyRegister {
        index: usize,
        position: Span,
    },
    ExpectedConcreteValue {
        found: AbstractValue,
        position: Span,
    },
    ExpectedValue {
        found: Register,
        position: Span,
    },
    RegisterIndexOutOfBounds {
        index: usize,
        position: Span,
    },

    // Local errors
    UndefinedLocal {
        local_index: u8,
        position: Span,
    },

    // Execution errors
    ExpectedBoolean {
        found: Value,
        position: Span,
    },
    ExpectedFunction {
        found: ConcreteValue,
        position: Span,
    },
    ExpectedParent {
        position: Span,
    },
    ValueDisplay {
        error: io::ErrorKind,
        position: Span,
    },

    // Chunk errors
    ConstantIndexOutOfBounds {
        index: usize,
        position: Span,
    },
    InstructionIndexOutOfBounds {
        index: usize,
        position: Span,
    },
    LocalIndexOutOfBounds {
        index: usize,
        position: Span,
    },

    // Wrappers for foreign errors
    NativeFunction(NativeFunctionError),
    Value {
        error: ValueError,
        position: Span,
    },
}

impl AnnotatedError for VmError {
    fn title() -> &'static str {
        "Runtime Error"
    }

    fn description(&self) -> &'static str {
        match self {
            Self::ConstantIndexOutOfBounds { .. } => "Constant index out of bounds",
            Self::EmptyRegister { .. } => "Empty register",
            Self::ExpectedBoolean { .. } => "Expected boolean",
            Self::ExpectedConcreteValue { .. } => "Expected concrete value",
            Self::ExpectedFunction { .. } => "Expected function",
            Self::ExpectedParent { .. } => "Expected parent",
            Self::ExpectedValue { .. } => "Expected value",
            Self::InstructionIndexOutOfBounds { .. } => "Instruction index out of bounds",
            Self::LocalIndexOutOfBounds { .. } => "Local index out of bounds",
            Self::NativeFunction(error) => error.description(),
            Self::RegisterIndexOutOfBounds { .. } => "Register index out of bounds",
            Self::StackOverflow { .. } => "Stack overflow",
            Self::StackUnderflow { .. } => "Stack underflow",
            Self::UndefinedLocal { .. } => "Undefined local",
            Self::Value { .. } => "Value error",
            Self::ValueDisplay { .. } => "Value display error",
        }
    }

    fn detail_snippets(&self) -> SmallVec<[(String, Span); 2]> {
        match self {
            VmError::StackOverflow { position } => todo!(),
            VmError::StackUnderflow { position } => todo!(),
            VmError::EmptyRegister { index, position } => todo!(),
            VmError::ExpectedConcreteValue { found, position } => todo!(),
            VmError::ExpectedValue { found, position } => todo!(),
            VmError::RegisterIndexOutOfBounds { index, position } => todo!(),
            VmError::UndefinedLocal {
                local_index,
                position,
            } => todo!(),
            VmError::ExpectedBoolean { found, position } => todo!(),
            VmError::ExpectedFunction { found, position } => todo!(),
            VmError::ExpectedParent { position } => todo!(),
            VmError::ValueDisplay { error, position } => todo!(),
            VmError::ConstantIndexOutOfBounds { index, position } => todo!(),
            VmError::InstructionIndexOutOfBounds { index, position } => todo!(),
            VmError::LocalIndexOutOfBounds { index, position } => todo!(),
            VmError::NativeFunction(native_function_error) => todo!(),
            VmError::Value { error, position } => todo!(),
        }
    }

    fn help_snippets(&self) -> SmallVec<[(String, Span); 2]> {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use runners::{RunnerLogic, RUNNER_LOGIC_TABLE};

    use super::*;

    const ALL_OPERATIONS: [(Operation, RunnerLogic); 24] = [
        (Operation::MOVE, runners::r#move),
        (Operation::CLOSE, runners::close),
        (Operation::LOAD_BOOLEAN, runners::load_boolean),
        (Operation::LOAD_CONSTANT, runners::load_constant),
        (Operation::LOAD_LIST, runners::load_list),
        (Operation::LOAD_SELF, runners::load_self),
        (Operation::GET_LOCAL, runners::get_local),
        (Operation::SET_LOCAL, runners::set_local),
        (Operation::ADD, runners::add),
        (Operation::SUBTRACT, runners::subtract),
        (Operation::MULTIPLY, runners::multiply),
        (Operation::DIVIDE, runners::divide),
        (Operation::MODULO, runners::modulo),
        (Operation::TEST, runners::test),
        (Operation::TEST_SET, runners::test_set),
        (Operation::EQUAL, runners::equal),
        (Operation::LESS, runners::less),
        (Operation::LESS_EQUAL, runners::less_equal),
        (Operation::NEGATE, runners::negate),
        (Operation::NOT, runners::not),
        (Operation::CALL, runners::call),
        (Operation::CALL_NATIVE, runners::call_native),
        (Operation::JUMP, runners::jump),
        (Operation::RETURN, runners::r#return),
    ];

    #[test]
    fn operations_map_to_the_correct_runner() {
        for (operation, expected_runner) in ALL_OPERATIONS {
            let actual_runner = RUNNER_LOGIC_TABLE[operation.0 as usize];

            assert_eq!(
                expected_runner, actual_runner,
                "{operation} runner is incorrect"
            );
        }
    }
}
