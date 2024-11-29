//! Virtual machine and errors
use std::{
    cmp::Ordering,
    fmt::{self, Display, Formatter},
    io,
};

use crate::{
    compile, instruction::*, AbstractValue, AnnotatedError, Argument, Chunk, ConcreteValue,
    Destination, DustError, Instruction, NativeFunctionError, Operation, Span, Value, ValueError,
    ValueRef,
};

pub fn run(source: &str) -> Result<Option<ConcreteValue>, DustError> {
    let chunk = compile(source)?;
    let mut vm = Vm::new(&chunk, None);

    vm.run()
        .map_err(|error| DustError::Runtime { error, source })
}

/// Dust virtual machine.
///
/// See the [module-level documentation](index.html) for more information.
#[derive(Debug)]
pub struct Vm<'a> {
    chunk: &'a Chunk,
    stack: Vec<Register>,
    parent: Option<&'a Vm<'a>>,
    local_definitions: Vec<Option<u16>>,

    ip: usize,
    last_assigned_register: Option<u16>,
    current_position: Span,
}

impl<'a> Vm<'a> {
    const STACK_LIMIT: usize = u16::MAX as usize;

    pub fn new(chunk: &'a Chunk, parent: Option<&'a Vm<'a>>) -> Self {
        Self {
            chunk,
            stack: Vec::new(),
            parent,
            local_definitions: vec![None; chunk.locals().len()],
            ip: 0,
            last_assigned_register: None,
            current_position: Span(0, 0),
        }
    }

    pub fn chunk(&self) -> &Chunk {
        self.chunk
    }

    pub fn current_position(&self) -> Span {
        self.current_position
    }

    pub fn run(&mut self) -> Result<Option<ConcreteValue>, VmError> {
        while let Ok(instruction) = self.read() {
            log::info!(
                "{} | {} | {} | {}",
                self.ip - 1,
                self.current_position,
                instruction.operation(),
                instruction.disassembly_info()
            );

            match instruction.operation() {
                Operation::Move => {
                    let Move { from, to } = Move::from(&instruction);
                    let from_register_has_value = self
                        .stack
                        .get(from as usize)
                        .is_some_and(|register| !matches!(register, Register::Empty));
                    let register = Register::Pointer(Pointer::Stack(from));

                    if from_register_has_value {
                        self.set_register(to, register)?;
                    }
                }
                Operation::Close => {
                    let Close { from, to } = Close::from(&instruction);

                    if self.stack.len() < to as usize {
                        return Err(VmError::StackUnderflow {
                            position: self.current_position,
                        });
                    }

                    for register_index in from..to {
                        self.stack[register_index as usize] = Register::Empty;
                    }
                }
                Operation::LoadBoolean => {
                    let LoadBoolean {
                        destination,
                        value,
                        jump_next,
                    } = LoadBoolean::from(&instruction);
                    let register_index = self.get_destination(destination)?;
                    let boolean = ConcreteValue::Boolean(value).to_value();
                    let register = Register::Value(boolean);

                    self.set_register(register_index, register)?;

                    if jump_next {
                        self.jump(1, true);
                    }
                }
                Operation::LoadConstant => {
                    let LoadConstant {
                        destination,
                        constant_index,
                        jump_next,
                    } = LoadConstant::from(&instruction);
                    let register_index = self.get_destination(destination)?;
                    let register = Register::Pointer(Pointer::Constant(constant_index));

                    self.set_register(register_index, register)?;

                    if jump_next {
                        self.jump(1, true);
                    }
                }
                Operation::LoadList => {
                    let LoadList {
                        destination,
                        start_register,
                    } = LoadList::from(&instruction);
                    let register_index = self.get_destination(destination)?;
                    let mut pointers = Vec::new();

                    for register in start_register..register_index {
                        if let Some(Register::Empty) = self.stack.get(register as usize) {
                            continue;
                        }

                        let pointer = Pointer::Stack(register);

                        pointers.push(pointer);
                    }

                    let register =
                        Register::Value(AbstractValue::List { items: pointers }.to_value());

                    self.set_register(register_index, register)?;
                }
                Operation::LoadSelf => {
                    let LoadSelf { destination } = LoadSelf::from(&instruction);
                    let register_index = self.get_destination(destination)?;
                    let register = Register::Value(AbstractValue::FunctionSelf.to_value());

                    self.set_register(register_index, register)?;
                }
                Operation::DefineLocal => {
                    let DefineLocal {
                        register,
                        local_index,
                        is_mutable,
                    } = DefineLocal::from(&instruction);

                    self.local_definitions[local_index as usize] = Some(register);
                }
                Operation::GetLocal => {
                    let GetLocal {
                        destination,
                        local_index,
                    } = GetLocal::from(&instruction);
                    let register_index = self.get_destination(destination)?;
                    let local_register = self.local_definitions[local_index as usize].ok_or(
                        VmError::UndefinedLocal {
                            local_index,
                            position: self.current_position,
                        },
                    )?;
                    let register = Register::Pointer(Pointer::Stack(local_register));

                    self.set_register(register_index, register)?;
                }
                Operation::SetLocal => {
                    let SetLocal {
                        register,
                        local_index,
                    } = SetLocal::from(&instruction);

                    self.local_definitions[local_index as usize] = Some(register);
                }
                Operation::Add => {
                    let Add {
                        destination,
                        left,
                        right,
                    } = Add::from(&instruction);
                    let register_index = self.get_destination(destination)?;
                    let left = self.get_argument(left)?;
                    let right = self.get_argument(right)?;
                    let sum = left.add(right).map_err(|error| VmError::Value {
                        error,
                        position: self.current_position,
                    })?;

                    self.set_register(register_index, Register::Value(sum))?;
                }
                Operation::Subtract => {
                    let Subtract {
                        destination,
                        left,
                        right,
                    } = Subtract::from(&instruction);
                    let register_index = self.get_destination(destination)?;
                    let left = self.get_argument(left)?;
                    let right = self.get_argument(right)?;
                    let difference = left.subtract(right).map_err(|error| VmError::Value {
                        error,
                        position: self.current_position,
                    })?;

                    self.set_register(register_index, Register::Value(difference))?;
                }
                Operation::Multiply => {
                    let Multiply {
                        destination,
                        left,
                        right,
                    } = Multiply::from(&instruction);
                    let register_index = self.get_destination(destination)?;
                    let left = self.get_argument(left)?;
                    let right = self.get_argument(right)?;
                    let product = left.multiply(right).map_err(|error| VmError::Value {
                        error,
                        position: self.current_position,
                    })?;

                    self.set_register(register_index, Register::Value(product))?;
                }
                Operation::Divide => {
                    let Divide {
                        destination,
                        left,
                        right,
                    } = Divide::from(&instruction);
                    let register_index = self.get_destination(destination)?;
                    let left = self.get_argument(left)?;
                    let right = self.get_argument(right)?;
                    let quotient = left.divide(right).map_err(|error| VmError::Value {
                        error,
                        position: self.current_position,
                    })?;

                    self.set_register(register_index, Register::Value(quotient))?;
                }
                Operation::Modulo => {
                    let Modulo {
                        destination,
                        left,
                        right,
                    } = Modulo::from(&instruction);
                    let register_index = self.get_destination(destination)?;
                    let left = self.get_argument(left)?;
                    let right = self.get_argument(right)?;
                    let remainder = left.modulo(right).map_err(|error| VmError::Value {
                        error,
                        position: self.current_position,
                    })?;

                    self.set_register(register_index, Register::Value(remainder))?;
                }
                Operation::Test => {
                    let Test {
                        argument,
                        test_value,
                    } = Test::from(&instruction);
                    let value = self.get_argument(argument)?;
                    let boolean = if let ValueRef::Concrete(ConcreteValue::Boolean(boolean)) = value
                    {
                        *boolean
                    } else {
                        return Err(VmError::ExpectedBoolean {
                            found: value.to_owned(),
                            position: self.current_position,
                        });
                    };

                    if boolean == test_value {
                        self.jump(1, true);
                    }
                }
                Operation::TestSet => {
                    let TestSet {
                        destination,
                        argument,
                        test_value,
                    } = TestSet::from(&instruction);
                    let register_index = self.get_destination(destination)?;
                    let value = self.get_argument(argument)?;
                    let boolean = if let ValueRef::Concrete(ConcreteValue::Boolean(boolean)) = value
                    {
                        *boolean
                    } else {
                        return Err(VmError::ExpectedBoolean {
                            found: value.to_owned(),
                            position: self.current_position,
                        });
                    };

                    if boolean == test_value {
                        self.jump(1, true);
                    } else {
                        let pointer = match argument {
                            Argument::Constant(constant_index) => Pointer::Constant(constant_index),
                            Argument::Local(local_index) => {
                                let register_index = self.local_definitions[local_index as usize]
                                    .ok_or(VmError::UndefinedLocal {
                                    local_index,
                                    position: self.current_position,
                                })?;

                                Pointer::Stack(register_index)
                            }
                            Argument::Register(register_index) => Pointer::Stack(register_index),
                        };
                        let register = Register::Pointer(pointer);

                        self.set_register(register_index, register)?;
                    }
                }
                Operation::Equal => {
                    let Equal { value, left, right } = Equal::from(&instruction);
                    let left = self.get_argument(left)?;
                    let right = self.get_argument(right)?;
                    let equal_result = left.equal(right).map_err(|error| VmError::Value {
                        error,
                        position: self.current_position,
                    })?;
                    let is_equal =
                        if let Value::Concrete(ConcreteValue::Boolean(boolean)) = equal_result {
                            boolean
                        } else {
                            return Err(VmError::ExpectedBoolean {
                                found: equal_result,
                                position: self.current_position,
                            });
                        };

                    if is_equal == value {
                        self.jump(1, true);
                    }
                }
                Operation::Less => {
                    let Less { value, left, right } = Less::from(&instruction);
                    let left = self.get_argument(left)?;
                    let right = self.get_argument(right)?;
                    let less_result = left.less_than(right).map_err(|error| VmError::Value {
                        error,
                        position: self.current_position,
                    })?;
                    let is_less_than =
                        if let Value::Concrete(ConcreteValue::Boolean(boolean)) = less_result {
                            boolean
                        } else {
                            return Err(VmError::ExpectedBoolean {
                                found: less_result,
                                position: self.current_position,
                            });
                        };

                    if is_less_than == value {
                        self.jump(1, true);
                    }
                }
                Operation::LessEqual => {
                    let LessEqual { value, left, right } = LessEqual::from(&instruction);
                    let left = self.get_argument(left)?;
                    let right = self.get_argument(right)?;
                    let less_or_equal_result =
                        left.less_than_or_equal(right)
                            .map_err(|error| VmError::Value {
                                error,
                                position: self.current_position,
                            })?;
                    let is_less_than_or_equal =
                        if let Value::Concrete(ConcreteValue::Boolean(boolean)) =
                            less_or_equal_result
                        {
                            boolean
                        } else {
                            return Err(VmError::ExpectedBoolean {
                                found: less_or_equal_result,
                                position: self.current_position,
                            });
                        };

                    if is_less_than_or_equal == value {
                        self.jump(1, true);
                    }
                }
                Operation::Negate => {
                    let Negate {
                        destination,
                        argument,
                    } = Negate::from(&instruction);
                    let value = self.get_argument(argument)?;
                    let negated = value.negate().map_err(|error| VmError::Value {
                        error,
                        position: self.current_position,
                    })?;
                    let register_index = self.get_destination(destination)?;
                    let register = Register::Value(negated);

                    self.set_register(register_index, register)?;
                }
                Operation::Not => {
                    let Not {
                        destination,
                        argument,
                    } = Not::from(&instruction);
                    let value = self.get_argument(argument)?;
                    let not = value.not().map_err(|error| VmError::Value {
                        error,
                        position: self.current_position,
                    })?;
                    let register_index = self.get_destination(destination)?;
                    let register = Register::Value(not);

                    self.set_register(register_index, register)?;
                }
                Operation::Jump => {
                    let Jump {
                        offset,
                        is_positive,
                    } = Jump::from(&instruction);

                    self.jump(offset as usize, is_positive);
                }
                Operation::Call => {
                    let Call {
                        destination,
                        function,
                        argument_count,
                    } = Call::from(&instruction);
                    let register_index = self.get_destination(destination)?;
                    let function = self.get_argument(function)?;
                    let chunk = if let ValueRef::Concrete(ConcreteValue::Function(chunk)) = function
                    {
                        chunk
                    } else if let ValueRef::Abstract(AbstractValue::FunctionSelf) = function {
                        self.chunk
                    } else {
                        return Err(VmError::ExpectedFunction {
                            found: function.to_concrete_owned(self)?,
                            position: self.current_position,
                        });
                    };
                    let mut function_vm = Vm::new(chunk, Some(self));
                    let first_argument_index = register_index - argument_count;

                    for (argument_index, argument_register_index) in
                        (first_argument_index..register_index).enumerate()
                    {
                        function_vm.set_register(
                            argument_index as u16,
                            Register::Pointer(Pointer::ParentStack(argument_register_index)),
                        )?;

                        function_vm.local_definitions[argument_index] = Some(argument_index as u16);
                    }

                    let return_value = function_vm.run()?;

                    if let Some(concrete_value) = return_value {
                        let register = Register::Value(concrete_value.to_value());

                        self.set_register(register_index, register)?;
                    }
                }
                Operation::CallNative => {
                    let CallNative {
                        destination,
                        function,
                        argument_count,
                    } = CallNative::from(&instruction);
                    let return_value = function.call(self, instruction)?;

                    if let Some(value) = return_value {
                        let register_index = self.get_destination(destination)?;
                        let register = Register::Value(value);

                        self.set_register(register_index, register)?;
                    }
                }
                Operation::Return => {
                    let Return {
                        should_return_value,
                    } = Return::from(&instruction);

                    if !should_return_value {
                        return Ok(None);
                    }

                    return if let Some(register_index) = self.last_assigned_register {
                        let return_value = self
                            .open_register(register_index)?
                            .to_concrete_owned(self)?;

                        Ok(Some(return_value))
                    } else {
                        Err(VmError::StackUnderflow {
                            position: self.current_position,
                        })
                    };
                }
            }
        }

        Ok(None)
    }

    pub(crate) fn follow_pointer(&self, pointer: Pointer) -> Result<ValueRef, VmError> {
        match pointer {
            Pointer::Stack(register_index) => self.open_register(register_index),
            Pointer::Constant(constant_index) => {
                let constant = self.get_constant(constant_index)?;

                Ok(ValueRef::Concrete(constant))
            }
            Pointer::ParentStack(register_index) => {
                let parent = self
                    .parent
                    .as_ref()
                    .ok_or_else(|| VmError::ExpectedParent {
                        position: self.current_position,
                    })?;

                parent.open_register(register_index)
            }
            Pointer::ParentConstant(constant_index) => {
                let parent = self
                    .parent
                    .as_ref()
                    .ok_or_else(|| VmError::ExpectedParent {
                        position: self.current_position,
                    })?;
                let constant = parent.get_constant(constant_index)?;

                Ok(ValueRef::Concrete(constant))
            }
        }
    }

    fn open_register(&self, register_index: u16) -> Result<ValueRef, VmError> {
        let register_index = register_index as usize;
        let register =
            self.stack
                .get(register_index)
                .ok_or_else(|| VmError::RegisterIndexOutOfBounds {
                    index: register_index,
                    position: self.current_position,
                })?;

        log::trace!("Open R{register_index} to {register}");

        match register {
            Register::Value(value) => Ok(value.to_ref()),
            Register::Pointer(pointer) => self.follow_pointer(*pointer),
            Register::Empty => Err(VmError::EmptyRegister {
                index: register_index,
                position: self.current_position,
            }),
        }
    }

    pub(crate) fn open_register_allow_empty(
        &self,
        register_index: u16,
    ) -> Result<Option<ValueRef>, VmError> {
        let register_index = register_index as usize;
        let register =
            self.stack
                .get(register_index)
                .ok_or_else(|| VmError::RegisterIndexOutOfBounds {
                    index: register_index,
                    position: self.current_position,
                })?;

        log::trace!("Open R{register_index} to {register}");

        match register {
            Register::Value(value) => Ok(Some(value.to_ref())),
            Register::Pointer(pointer) => self.follow_pointer(*pointer).map(Some),
            Register::Empty => Ok(None),
        }
    }

    /// DRY helper for handling JUMP instructions
    fn jump(&mut self, offset: usize, is_positive: bool) {
        log::trace!(
            "Jumping {}",
            if is_positive {
                format!("+{}", offset)
            } else {
                format!("-{}", offset)
            }
        );

        let new_ip = if is_positive {
            self.ip + offset
        } else {
            self.ip - offset - 1
        };
        self.ip = new_ip;
    }

    /// DRY helper to get a register index from a Destination
    fn get_destination(&self, destination: Destination) -> Result<u16, VmError> {
        let index = match destination {
            Destination::Register(register_index) => register_index,
            Destination::Local(local_index) => self
                .local_definitions
                .get(local_index as usize)
                .copied()
                .flatten()
                .ok_or_else(|| VmError::UndefinedLocal {
                    local_index,
                    position: self.current_position,
                })?,
        };

        Ok(index)
    }

    /// DRY helper to get a value from an Argument
    fn get_argument(&self, argument: Argument) -> Result<ValueRef, VmError> {
        let value_ref = match argument {
            Argument::Constant(constant_index) => {
                ValueRef::Concrete(self.get_constant(constant_index)?)
            }
            Argument::Register(register_index) => self.open_register(register_index)?,
            Argument::Local(local_index) => self.get_local(local_index)?,
        };

        Ok(value_ref)
    }

    fn set_register(&mut self, to_register: u16, register: Register) -> Result<(), VmError> {
        self.last_assigned_register = Some(to_register);

        let length = self.stack.len();
        let to_register = to_register as usize;

        if length == Self::STACK_LIMIT {
            return Err(VmError::StackOverflow {
                position: self.current_position,
            });
        }

        match to_register.cmp(&length) {
            Ordering::Less => {
                log::trace!("Change R{to_register} to {register}");

                self.stack[to_register] = register;

                Ok(())
            }
            Ordering::Equal => {
                log::trace!("Set R{to_register} to {register}");

                self.stack.push(register);

                Ok(())
            }
            Ordering::Greater => {
                let difference = to_register - length;

                for index in 0..difference {
                    log::trace!("Set R{index} to {register}");

                    self.stack.push(Register::Empty);
                }

                log::trace!("Set R{to_register} to {register}");

                self.stack.push(register);

                Ok(())
            }
        }
    }

    fn get_constant(&self, constant_index: u16) -> Result<&ConcreteValue, VmError> {
        self.chunk
            .constants()
            .get(constant_index as usize)
            .ok_or_else(|| VmError::ConstantIndexOutOfBounds {
                index: constant_index as usize,
                position: self.current_position,
            })
    }

    fn get_local(&self, local_index: u16) -> Result<ValueRef, VmError> {
        let register_index = self
            .local_definitions
            .get(local_index as usize)
            .ok_or_else(|| VmError::UndefinedLocal {
                local_index,
                position: self.current_position,
            })?
            .ok_or_else(|| VmError::UndefinedLocal {
                local_index,
                position: self.current_position,
            })?;

        self.open_register(register_index)
    }

    fn read(&mut self) -> Result<Instruction, VmError> {
        let (instruction, _type, position) =
            self.chunk.instructions().get(self.ip).ok_or_else(|| {
                VmError::InstructionIndexOutOfBounds {
                    index: self.ip,
                    position: self.current_position,
                }
            })?;

        self.ip += 1;
        self.current_position = *position;

        Ok(*instruction)
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
    Stack(u16),
    Constant(u16),
    ParentStack(u16),
    ParentConstant(u16),
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
        local_index: u16,
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

    fn details(&self) -> Option<String> {
        match self {
            Self::EmptyRegister { index, .. } => Some(format!("Register R{index} is empty")),
            Self::ExpectedFunction { found, .. } => Some(format!("{found} is not a function")),

            Self::RegisterIndexOutOfBounds { index, .. } => {
                Some(format!("Register {index} does not exist"))
            }
            Self::NativeFunction(error) => error.details(),
            Self::Value { error, .. } => Some(error.to_string()),
            Self::ValueDisplay { error, .. } => Some(error.to_string() + " while displaying value"),
            _ => None,
        }
    }

    fn position(&self) -> Span {
        match self {
            Self::ConstantIndexOutOfBounds { position, .. } => *position,
            Self::EmptyRegister { position, .. } => *position,
            Self::ExpectedBoolean { position, .. } => *position,
            Self::ExpectedConcreteValue { position, .. } => *position,
            Self::ExpectedFunction { position, .. } => *position,
            Self::ExpectedParent { position } => *position,
            Self::ExpectedValue { position, .. } => *position,
            Self::InstructionIndexOutOfBounds { position, .. } => *position,
            Self::LocalIndexOutOfBounds { position, .. } => *position,
            Self::NativeFunction(error) => error.position(),
            Self::RegisterIndexOutOfBounds { position, .. } => *position,
            Self::StackOverflow { position } => *position,
            Self::StackUnderflow { position } => *position,
            Self::UndefinedLocal { position, .. } => *position,
            Self::Value { position, .. } => *position,
            Self::ValueDisplay { position, .. } => *position,
        }
    }
}
