//! Virtual machine and errors
use std::{
    fmt::{self, Display, Formatter},
    io,
};

use smallvec::{smallvec, SmallVec};

use crate::{
    compile, instruction::*, AbstractValue, AnnotatedError, Argument, Chunk, ConcreteValue,
    DustError, Instruction, NativeFunctionError, Operation, Span, Value, ValueError, ValueRef,
};

pub fn run(source: &str) -> Result<Option<ConcreteValue>, DustError> {
    let chunk = compile(source)?;
    let mut vm = Vm::new(source, &chunk, None);

    vm.run().map_err(|error| DustError::runtime(error, source))
}

/// Dust virtual machine.
///
/// See the [module-level documentation](index.html) for more information.
#[derive(Debug)]
pub struct Vm<'a> {
    stack: SmallVec<[Register; 64]>,

    chunk: &'a Chunk,
    parent: Option<&'a Vm<'a>>,

    ip: usize,
    last_assigned_register: Option<u8>,
    source: &'a str,
}

impl<'a> Vm<'a> {
    pub fn new(source: &'a str, chunk: &'a Chunk, parent: Option<&'a Vm<'a>>) -> Self {
        Self {
            chunk,
            stack: smallvec![Register::Empty; chunk.stack_size()],
            parent,
            ip: 0,
            last_assigned_register: None,
            source,
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

    pub fn run(&mut self) -> Result<Option<ConcreteValue>, VmError> {
        loop {
            let instruction = self.read();

            log::info!(
                "{} | {} | {} | {}",
                self.ip - 1,
                self.current_position(),
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
                            position: self.current_position(),
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
                    let boolean = ConcreteValue::Boolean(value).to_value();
                    let register = Register::Value(boolean);

                    self.set_register(destination, register)?;

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
                    let register = Register::Pointer(Pointer::Constant(constant_index));

                    self.set_register(destination, register)?;

                    if jump_next {
                        self.jump(1, true);
                    }
                }
                Operation::LoadList => {
                    let LoadList {
                        destination,
                        start_register,
                    } = LoadList::from(&instruction);
                    let mut pointers = Vec::new();

                    for register in start_register..destination {
                        if let Some(Register::Empty) = self.stack.get(register as usize) {
                            continue;
                        }

                        let pointer = Pointer::Stack(register);

                        pointers.push(pointer);
                    }

                    let register = Register::Value(
                        AbstractValue::List {
                            item_pointers: pointers,
                        }
                        .to_value(),
                    );

                    self.set_register(destination, register)?;
                }
                Operation::LoadSelf => {
                    let LoadSelf { destination } = LoadSelf::from(&instruction);
                    let register = Register::Value(AbstractValue::FunctionSelf.to_value());

                    self.set_register(destination, register)?;
                }
                Operation::GetLocal => {
                    let GetLocal {
                        destination,
                        local_index,
                    } = GetLocal::from(&instruction);
                    let local_register = self.get_local_register(local_index)?;
                    let register = Register::Pointer(Pointer::Stack(local_register));

                    self.set_register(destination, register)?;
                }
                Operation::SetLocal => {
                    let SetLocal {
                        register_index,
                        local_index,
                    } = SetLocal::from(&instruction);
                    let local_register_index = self.get_local_register(local_index)?;
                    let register = Register::Pointer(Pointer::Stack(register_index));

                    self.set_register(local_register_index, register)?;
                }
                Operation::Add => {
                    let Add {
                        destination,
                        left,
                        right,
                    } = Add::from(&instruction);
                    let left = self.get_argument(left)?;
                    let right = self.get_argument(right)?;
                    let sum_result = left.add(right);
                    let sum = match sum_result {
                        Ok(sum) => sum,
                        Err(error) => {
                            return Err(VmError::Value {
                                error,
                                position: self.current_position(),
                            });
                        }
                    };

                    self.set_register(destination, Register::Value(sum))?;
                }
                Operation::Subtract => {
                    let Subtract {
                        destination,
                        left,
                        right,
                    } = Subtract::from(&instruction);
                    let left = self.get_argument(left)?;
                    let right = self.get_argument(right)?;
                    let subtraction_result = left.subtract(right);
                    let difference = match subtraction_result {
                        Ok(difference) => difference,
                        Err(error) => {
                            return Err(VmError::Value {
                                error,
                                position: self.current_position(),
                            });
                        }
                    };

                    self.set_register(destination, Register::Value(difference))?;
                }
                Operation::Multiply => {
                    let Multiply {
                        destination,
                        left,
                        right,
                    } = Multiply::from(&instruction);
                    let left = self.get_argument(left)?;
                    let right = self.get_argument(right)?;
                    let multiplication_result = left.multiply(right);
                    let product = match multiplication_result {
                        Ok(product) => product,
                        Err(error) => {
                            return Err(VmError::Value {
                                error,
                                position: self.current_position(),
                            });
                        }
                    };

                    self.set_register(destination, Register::Value(product))?;
                }
                Operation::Divide => {
                    let Divide {
                        destination,
                        left,
                        right,
                    } = Divide::from(&instruction);
                    let left = self.get_argument(left)?;
                    let right = self.get_argument(right)?;
                    let division_result = left.divide(right);
                    let quotient = match division_result {
                        Ok(quotient) => quotient,
                        Err(error) => {
                            return Err(VmError::Value {
                                error,
                                position: self.current_position(),
                            });
                        }
                    };

                    self.set_register(destination, Register::Value(quotient))?;
                }
                Operation::Modulo => {
                    let Modulo {
                        destination,
                        left,
                        right,
                    } = Modulo::from(&instruction);
                    let left = self.get_argument(left)?;
                    let right = self.get_argument(right)?;
                    let modulo_result = left.modulo(right);
                    let remainder = match modulo_result {
                        Ok(remainder) => remainder,
                        Err(error) => {
                            return Err(VmError::Value {
                                error,
                                position: self.current_position(),
                            });
                        }
                    };

                    self.set_register(destination, Register::Value(remainder))?;
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
                            position: self.current_position(),
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
                    let value = self.get_argument(argument)?;
                    let boolean = if let ValueRef::Concrete(ConcreteValue::Boolean(boolean)) = value
                    {
                        *boolean
                    } else {
                        return Err(VmError::ExpectedBoolean {
                            found: value.to_owned(),
                            position: self.current_position(),
                        });
                    };

                    if boolean == test_value {
                        self.jump(1, true);
                    } else {
                        let pointer = match argument {
                            Argument::Constant(constant_index) => Pointer::Constant(constant_index),
                            Argument::Register(register_index) => Pointer::Stack(register_index),
                        };
                        let register = Register::Pointer(pointer);

                        self.set_register(destination, register)?;
                    }
                }
                Operation::Equal => {
                    let Equal {
                        destination,
                        value,
                        left,
                        right,
                    } = Equal::from(&instruction);
                    let left = self.get_argument(left)?;
                    let right = self.get_argument(right)?;
                    let equal_result = left.equal(right).map_err(|error| VmError::Value {
                        error,
                        position: self.current_position(),
                    })?;
                    let is_equal =
                        if let Value::Concrete(ConcreteValue::Boolean(boolean)) = equal_result {
                            boolean
                        } else {
                            return Err(VmError::ExpectedBoolean {
                                found: equal_result,
                                position: self.current_position(),
                            });
                        };
                    let comparison = is_equal == value;
                    let register =
                        Register::Value(Value::Concrete(ConcreteValue::Boolean(comparison)));

                    self.set_register(destination, register)?;
                }
                Operation::Less => {
                    let Less {
                        destination,
                        value,
                        left,
                        right,
                    } = Less::from(&instruction);
                    let left = self.get_argument(left)?;
                    let right = self.get_argument(right)?;
                    let less_result = left.less_than(right);
                    let less_than_value = match less_result {
                        Ok(value) => value,
                        Err(error) => {
                            return Err(VmError::Value {
                                error,
                                position: self.current_position(),
                            });
                        }
                    };
                    let is_less_than = match less_than_value {
                        Value::Concrete(ConcreteValue::Boolean(boolean)) => boolean,
                        _ => {
                            return Err(VmError::ExpectedBoolean {
                                found: less_than_value,
                                position: self.current_position(),
                            });
                        }
                    };
                    let comparison = is_less_than == value;
                    let register =
                        Register::Value(Value::Concrete(ConcreteValue::Boolean(comparison)));

                    self.set_register(destination, register)?;
                }
                Operation::LessEqual => {
                    let LessEqual {
                        destination,
                        value,
                        left,
                        right,
                    } = LessEqual::from(&instruction);
                    let left = self.get_argument(left)?;
                    let right = self.get_argument(right)?;
                    let less_or_equal_result = left.less_than_or_equal(right);
                    let less_or_equal_value = match less_or_equal_result {
                        Ok(value) => value,
                        Err(error) => {
                            return Err(VmError::Value {
                                error,
                                position: self.current_position(),
                            });
                        }
                    };
                    let is_less_than_or_equal = match less_or_equal_value {
                        Value::Concrete(ConcreteValue::Boolean(boolean)) => boolean,
                        _ => {
                            return Err(VmError::ExpectedBoolean {
                                found: less_or_equal_value,
                                position: self.current_position(),
                            });
                        }
                    };
                    let comparison = is_less_than_or_equal == value;
                    let register =
                        Register::Value(Value::Concrete(ConcreteValue::Boolean(comparison)));

                    self.set_register(destination, register)?;
                }
                Operation::Negate => {
                    let Negate {
                        destination,
                        argument,
                    } = Negate::from(&instruction);
                    let value = self.get_argument(argument)?;
                    let negated = value.negate().map_err(|error| VmError::Value {
                        error,
                        position: self.current_position(),
                    })?;
                    let register = Register::Value(negated);

                    self.set_register(destination, register)?;
                }
                Operation::Not => {
                    let Not {
                        destination,
                        argument,
                    } = Not::from(&instruction);
                    let value = self.get_argument(argument)?;
                    let not = value.not().map_err(|error| VmError::Value {
                        error,
                        position: self.current_position(),
                    })?;
                    let register = Register::Value(not);

                    self.set_register(destination, register)?;
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
                    let function = self.get_argument(function)?;
                    let chunk = if let ValueRef::Concrete(ConcreteValue::Function(chunk)) = function
                    {
                        chunk
                    } else if let ValueRef::Abstract(AbstractValue::FunctionSelf) = function {
                        self.chunk
                    } else {
                        return Err(VmError::ExpectedFunction {
                            found: function.into_concrete_owned(self)?,
                            position: self.current_position(),
                        });
                    };
                    let mut function_vm = Vm::new(self.source, chunk, Some(self));
                    let first_argument_index = destination - argument_count;

                    for (argument_index, argument_register_index) in
                        (first_argument_index..destination).enumerate()
                    {
                        function_vm.set_register(
                            argument_index as u8,
                            Register::Pointer(Pointer::ParentStack(argument_register_index)),
                        )?;
                    }

                    let return_value = function_vm.run()?;

                    if let Some(concrete_value) = return_value {
                        let register = Register::Value(concrete_value.to_value());

                        self.set_register(destination, register)?;
                    }
                }
                Operation::CallNative => {
                    let CallNative {
                        destination,
                        function,
                        argument_count,
                    } = CallNative::from(&instruction);
                    let first_argument_index = (destination - argument_count) as usize;
                    let argument_range = first_argument_index..destination as usize;
                    let argument_registers = &self.stack[argument_range];
                    let mut arguments: SmallVec<[ValueRef; 4]> = SmallVec::new();

                    for register in argument_registers {
                        let value = match register {
                            Register::Value(value) => value.to_ref(),
                            Register::Pointer(pointer) => {
                                let value_option = self.follow_pointer_allow_empty(*pointer)?;

                                match value_option {
                                    Some(value) => value,
                                    None => continue,
                                }
                            }
                            Register::Empty => continue,
                        };

                        arguments.push(value);
                    }

                    let call_result = function.call(self, arguments);
                    let return_value = match call_result {
                        Ok(value_option) => value_option,
                        Err(error) => return Err(VmError::NativeFunction(error)),
                    };

                    if let Some(value) = return_value {
                        let register = Register::Value(value);

                        self.set_register(destination, register)?;
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
                            .into_concrete_owned(self)?;

                        Ok(Some(return_value))
                    } else {
                        Err(VmError::StackUnderflow {
                            position: self.current_position(),
                        })
                    };
                }
                _ => unreachable!(),
            }
        }
    }

    pub(crate) fn follow_pointer(&self, pointer: Pointer) -> Result<ValueRef, VmError> {
        match pointer {
            Pointer::Stack(register_index) => self.open_register(register_index),
            Pointer::Constant(constant_index) => {
                let constant = self.get_constant(constant_index);

                Ok(ValueRef::Concrete(constant))
            }
            Pointer::ParentStack(register_index) => {
                let parent = self
                    .parent
                    .as_ref()
                    .ok_or_else(|| VmError::ExpectedParent {
                        position: self.current_position(),
                    })?;

                parent.open_register(register_index)
            }
            Pointer::ParentConstant(constant_index) => {
                let parent = self
                    .parent
                    .as_ref()
                    .ok_or_else(|| VmError::ExpectedParent {
                        position: self.current_position(),
                    })?;
                let constant = parent.get_constant(constant_index);

                Ok(ValueRef::Concrete(constant))
            }
        }
    }

    pub(crate) fn follow_pointer_allow_empty(
        &self,
        pointer: Pointer,
    ) -> Result<Option<ValueRef>, VmError> {
        match pointer {
            Pointer::Stack(register_index) => self.open_register_allow_empty(register_index),
            Pointer::Constant(constant_index) => {
                let constant = self.get_constant(constant_index);

                Ok(Some(ValueRef::Concrete(constant)))
            }
            Pointer::ParentStack(register_index) => {
                let parent = self
                    .parent
                    .as_ref()
                    .ok_or_else(|| VmError::ExpectedParent {
                        position: self.current_position(),
                    })?;

                parent.open_register_allow_empty(register_index)
            }
            Pointer::ParentConstant(constant_index) => {
                let parent = self
                    .parent
                    .as_ref()
                    .ok_or_else(|| VmError::ExpectedParent {
                        position: self.current_position(),
                    })?;
                let constant = parent.get_constant(constant_index);

                Ok(Some(ValueRef::Concrete(constant)))
            }
        }
    }

    fn open_register(&self, register_index: u8) -> Result<ValueRef, VmError> {
        let register_index = register_index as usize;

        assert!(
            register_index < self.stack.len(),
            "VM Error: Register index out of bounds"
        );

        let register = &self.stack[register_index];

        log::trace!("Open R{register_index} to {register}");

        match register {
            Register::Value(value) => Ok(value.to_ref()),
            Register::Pointer(pointer) => self.follow_pointer(*pointer),
            Register::Empty => Err(VmError::EmptyRegister {
                index: register_index,
                position: self.current_position(),
            }),
        }
    }

    fn open_register_allow_empty(&self, register_index: u8) -> Result<Option<ValueRef>, VmError> {
        let register_index = register_index as usize;
        let register =
            self.stack
                .get(register_index)
                .ok_or_else(|| VmError::RegisterIndexOutOfBounds {
                    index: register_index,
                    position: self.current_position(),
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

    /// DRY helper to get a value from an Argument
    fn get_argument(&self, argument: Argument) -> Result<ValueRef, VmError> {
        let value_ref = match argument {
            Argument::Constant(constant_index) => {
                ValueRef::Concrete(self.get_constant(constant_index))
            }
            Argument::Register(register_index) => self.open_register(register_index)?,
        };

        Ok(value_ref)
    }

    #[inline(always)]
    fn set_register(&mut self, to_register: u8, register: Register) -> Result<(), VmError> {
        self.last_assigned_register = Some(to_register);

        let to_register = to_register as usize;

        assert!(
            self.stack.len() > to_register,
            "VM Error: Register index out of bounds"
        );

        self.stack[to_register] = register;

        Ok(())
    }

    #[inline(always)]
    fn get_constant(&self, constant_index: u8) -> &ConcreteValue {
        let constant_index = constant_index as usize;
        let constants = self.chunk.constants();

        assert!(
            constant_index < constants.len(),
            "VM Error: Constant index out of bounds"
        );

        &constants[constant_index]
    }

    #[inline(always)]
    fn get_local_register(&self, local_index: u8) -> Result<u8, VmError> {
        let local_index = local_index as usize;
        let locals = self.chunk.locals();

        assert!(
            local_index < locals.len(),
            "VM Error: Local index out of bounds"
        );

        let register_index = locals[local_index].register_index;

        Ok(register_index)
    }

    #[inline(always)]
    fn read(&mut self) -> Instruction {
        let instructions = self.chunk.instructions();

        assert!(
            self.ip < instructions.len(),
            "{}",
            DustError::runtime(
                VmError::InstructionIndexOutOfBounds {
                    index: self.ip,
                    position: self.current_position()
                },
                self.source,
            )
        );

        let (instruction, _) = instructions[self.ip];

        self.ip += 1;

        instruction
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
