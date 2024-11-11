//! Virtual machine and errors
use std::{
    cmp::Ordering,
    collections::HashMap,
    fmt::{self, Display, Formatter},
    mem::replace,
    rc::Weak,
};

use crate::{
    compile, value::ConcreteValue, AnnotatedError, Chunk, ChunkError, DustError, FunctionBorrowed,
    Instruction, NativeFunction, NativeFunctionError, Operation, Span, Type, Value, ValueError,
};

pub fn run(source: &str) -> Result<Option<ConcreteValue>, DustError> {
    let chunk = compile(source)?;
    let mut vm = Vm::new(&chunk, None);

    vm.run()
        .map_err(|error| DustError::Runtime { error, source })
}

pub fn run_and_display_output(source: &str) {
    match run(source) {
        Ok(Some(value)) => println!("{}", value),
        Ok(None) => {}
        Err(error) => eprintln!("{}", error.report()),
    }
}

/// Dust virtual machine.
///
/// See the [module-level documentation](index.html) for more information.
#[derive(Debug, Eq, PartialEq)]
pub struct Vm<'chunk, 'parent, 'stack> {
    ip: usize,
    chunk: &'chunk Chunk,
    stack: Vec<Register<'stack>>,
    local_definitions: HashMap<u8, u8>,
    last_assigned_register: Option<u8>,
    parent: Option<&'parent Vm<'chunk, 'stack, 'parent>>,
}

impl<'chunk, 'parent, 'stack: 'chunk + 'parent> Vm<'chunk, 'parent, 'stack> {
    const STACK_LIMIT: usize = u16::MAX as usize;

    pub fn new(chunk: &'chunk Chunk, parent: Option<&'parent Vm<'chunk, 'parent, 'stack>>) -> Self {
        Self {
            ip: 0,
            chunk,
            stack: Vec::new(),
            local_definitions: HashMap::new(),
            last_assigned_register: None,
            parent,
        }
    }

    pub fn run(&'stack mut self) -> Result<Option<ConcreteValue>, VmError> {
        while let Ok((instruction, position)) = self.read(Span(0, 0)).copied() {
            log::info!(
                "{} | {} | {} | {}",
                self.ip - 1,
                position,
                instruction.operation(),
                instruction.disassembly_info(self.chunk)
            );

            match instruction.operation() {
                Operation::Move => {
                    let to_register = instruction.a();
                    let from_register = instruction.b();
                    let from_register_has_value = self
                        .stack
                        .get(from_register as usize)
                        .is_some_and(|register| !matches!(register, Register::Empty));

                    if from_register_has_value {
                        self.set_register(
                            to_register,
                            Register::StackPointer(from_register),
                            position,
                        )?;
                    }
                }
                Operation::Close => {
                    let from_register = instruction.b();
                    let to_register = instruction.c();

                    if self.stack.len() < to_register as usize {
                        return Err(VmError::StackUnderflow { position });
                    }

                    for register_index in from_register..to_register {
                        self.stack[register_index as usize] = Register::Empty;
                    }
                }
                Operation::LoadBoolean => {
                    let to_register = instruction.a();
                    let boolean = instruction.b_as_boolean();
                    let jump = instruction.c_as_boolean();
                    let boolean = ConcreteValue::boolean(boolean);

                    self.set_register(to_register, Register::Value(boolean), position)?;

                    if jump {
                        self.ip += 1;
                    }
                }
                Operation::LoadConstant => {
                    let to_register = instruction.a();
                    let from_constant = instruction.b();
                    let jump = instruction.c_as_boolean();

                    self.set_register(
                        to_register,
                        Register::ConstantPointer(from_constant),
                        position,
                    )?;

                    if jump {
                        self.ip += 1
                    }
                }
                Operation::LoadList => {
                    let to_register = instruction.a();
                    let start_register = instruction.b();
                    let mut list = Vec::new();

                    for register_index in start_register..to_register {
                        let value = self.open_register(register_index, position)?;

                        list.push(value);
                    }

                    self.set_register(to_register, Register::List(list), position)?;
                }
                Operation::LoadSelf => {
                    let to_register = instruction.a();
                    let function = Value::FunctionBorrowed(FunctionBorrowed::new(self.chunk));

                    // self.set_register(to_register, Register::Value(function), position)?;

                    todo!()
                }
                Operation::DefineLocal => {
                    let from_register = instruction.a();
                    let to_local = instruction.b();

                    self.define_local(to_local, from_register)?;
                }
                Operation::GetLocal => {
                    let to_register = instruction.a();
                    let local_index = instruction.b();
                    let local_register = self.local_definitions.get(&local_index).copied().ok_or(
                        VmError::UndefinedLocal {
                            local_index,
                            position,
                        },
                    )?;

                    self.set_register(
                        to_register,
                        Register::StackPointer(local_register),
                        position,
                    )?;
                }
                Operation::SetLocal => {
                    let from_register = instruction.a();
                    let to_local = instruction.b();
                    let local_register = self.local_definitions.get(&to_local).copied().ok_or(
                        VmError::UndefinedLocal {
                            local_index: to_local,
                            position,
                        },
                    )?;

                    self.set_register(
                        local_register,
                        Register::StackPointer(from_register),
                        position,
                    )?;
                }
                Operation::Add => {
                    let to_register = instruction.a();
                    let (left, right) = self.get_arguments(instruction, position)?;
                    let sum = left
                        .add(&right)
                        .map_err(|error| VmError::Value { error, position })?;

                    self.set_register(to_register, Register::Value(sum), position)?;
                }
                Operation::Subtract => {
                    let to_register = instruction.a();
                    let (left, right) = self.get_arguments(instruction, position)?;
                    let difference = left
                        .subtract(&right)
                        .map_err(|error| VmError::Value { error, position })?;

                    self.set_register(to_register, Register::Value(difference), position)?;
                }
                Operation::Multiply => {
                    let to_register = instruction.a();
                    let (left, right) = self.get_arguments(instruction, position)?;
                    let product = left
                        .multiply(&right)
                        .map_err(|error| VmError::Value { error, position })?;

                    self.set_register(to_register, Register::Value(product), position)?;
                }
                Operation::Divide => {
                    let to_register = instruction.a();
                    let (left, right) = self.get_arguments(instruction, position)?;
                    let quotient = left
                        .divide(&right)
                        .map_err(|error| VmError::Value { error, position })?;

                    self.set_register(to_register, Register::Value(quotient), position)?;
                }
                Operation::Modulo => {
                    let to_register = instruction.a();
                    let (left, right) = self.get_arguments(instruction, position)?;
                    let remainder = left
                        .modulo(&right)
                        .map_err(|error| VmError::Value { error, position })?;

                    self.set_register(to_register, Register::Value(remainder), position)?;
                }
                Operation::Test => {
                    let register = instruction.a();
                    let test_value = instruction.c_as_boolean();
                    let value = self.open_register(register, position)?;
                    let boolean = if let Some(boolean) = value.as_boolean() {
                        boolean
                    } else {
                        return Err(VmError::ExpectedBoolean {
                            found: value.into_concrete(),
                            position,
                        });
                    };

                    if boolean != test_value {
                        self.ip += 1;
                    }
                }
                Operation::TestSet => todo!(),
                Operation::Equal => {
                    debug_assert_eq!(
                        self.get_instruction(self.ip, position)?.0.operation(),
                        Operation::Jump
                    );

                    let compare_to = instruction.a_as_boolean();
                    let (left, right) = self.get_arguments(instruction, position)?;
                    let equal_result = left
                        .equal(&right)
                        .map_err(|error| VmError::Value { error, position })?;
                    let is_equal = if let ConcreteValue::Boolean(boolean) = equal_result {
                        boolean
                    } else {
                        return Err(VmError::ExpectedBoolean {
                            found: equal_result.clone(),
                            position,
                        });
                    };

                    if is_equal == compare_to {
                        self.ip += 1;
                    } else {
                        let jump = self.get_instruction(self.ip, position)?.0;

                        self.jump(jump);
                    }
                }
                Operation::Less => {
                    debug_assert_eq!(
                        self.get_instruction(self.ip, position)?.0.operation(),
                        Operation::Jump
                    );

                    let compare_to = instruction.a_as_boolean();
                    let (left, right) = self.get_arguments(instruction, position)?;
                    let less_result = left
                        .less_than(&right)
                        .map_err(|error| VmError::Value { error, position })?;
                    let is_less_than = if let ConcreteValue::Boolean(boolean) = less_result {
                        boolean
                    } else {
                        return Err(VmError::ExpectedBoolean {
                            found: less_result.clone(),
                            position,
                        });
                    };

                    if is_less_than == compare_to {
                        self.ip += 1;
                    } else {
                        let jump = self.get_instruction(self.ip, position)?.0;

                        self.jump(jump);
                    }
                }
                Operation::LessEqual => {
                    debug_assert_eq!(
                        self.get_instruction(self.ip, position)?.0.operation(),
                        Operation::Jump
                    );

                    let compare_to = instruction.a_as_boolean();
                    let (left, right) = self.get_arguments(instruction, position)?;
                    let less_or_equal_result = left
                        .less_than_or_equal(&right)
                        .map_err(|error| VmError::Value { error, position })?;
                    let is_less_than_or_equal =
                        if let ConcreteValue::Boolean(boolean) = less_or_equal_result {
                            boolean
                        } else {
                            return Err(VmError::ExpectedBoolean {
                                found: less_or_equal_result.clone(),
                                position,
                            });
                        };

                    if is_less_than_or_equal == compare_to {
                        self.ip += 1;
                    } else {
                        let jump = self.get_instruction(self.ip, position)?.0;

                        self.jump(jump);
                    }
                }
                Operation::Negate => {
                    let value =
                        self.get_argument(instruction.b(), instruction.b_is_constant(), position)?;
                    let negated = value
                        .negate()
                        .map_err(|error| VmError::Value { error, position })?;

                    self.set_register(instruction.a(), Register::Value(negated), position)?;
                }
                Operation::Not => {
                    let value =
                        self.get_argument(instruction.b(), instruction.b_is_constant(), position)?;
                    let not = value
                        .not()
                        .map_err(|error| VmError::Value { error, position })?;

                    self.set_register(instruction.a(), Register::Value(not), position)?;
                }
                Operation::Jump => self.jump(instruction),
                Operation::Call => {
                    let to_register = instruction.a();
                    let function_register = instruction.b();
                    let argument_count = instruction.c();
                    let value = self.open_register(function_register, position)?;
                    let function = if let Some(function) = value.as_function() {
                        function
                    } else {
                        return Err(VmError::ExpectedFunction {
                            found: value.into_concrete(),
                            position,
                        });
                    };
                    let mut function_vm = Vm::new(function.chunk(), Some(self));

                    let first_argument_index = function_register + 1;

                    for argument_index in
                        first_argument_index..first_argument_index + argument_count
                    {
                        let top_of_stack = function_vm.stack.len() as u8;

                        function_vm.set_register(
                            top_of_stack,
                            Register::ParentStackPointer(argument_index),
                            position,
                        )?
                    }

                    let return_value = function_vm.run()?;

                    if let Some(value) = return_value {
                        self.set_register(to_register, Register::Value(value), position)?;
                    }
                }
                Operation::CallNative => {
                    let native_function = NativeFunction::from(instruction.b());
                    let return_value = native_function.call(self, instruction, position)?;

                    if let Some(concrete_value) = return_value {
                        let to_register = instruction.a();

                        self.set_register(to_register, Register::Value(concrete_value), position)?;
                    }
                }
                Operation::Return => {
                    let should_return_value = instruction.b_as_boolean();

                    if !should_return_value {
                        return Ok(None);
                    }

                    return if let Some(register_index) = self.last_assigned_register {
                        self.open_register(register_index, position)
                            .map(|value| Some(value.into_concrete()))
                    } else {
                        Err(VmError::StackUnderflow { position })
                    };
                }
            }
        }

        Ok(None)
    }

    pub(crate) fn open_register(
        &'stack self,
        register_index: u8,
        position: Span,
    ) -> Result<&'stack ConcreteValue, VmError> {
        let register_index = register_index as usize;
        let register =
            self.stack
                .get(register_index)
                .ok_or_else(|| VmError::RegisterIndexOutOfBounds {
                    index: register_index,
                    position,
                })?;

        log::trace!("Open R{register_index} to {register}");

        match register {
            Register::Value(value) => Ok(value),
            Register::List(list) => Ok(ConcreteValue::List(
                list.into_iter()
                    .map(|concrete_value| (*concrete_value).clone())
                    .collect(),
            )),
            Register::StackPointer(register_index) => self.open_register(*register_index, position),
            Register::ConstantPointer(constant_index) => {
                self.get_constant(*constant_index, position)
            }
            Register::ParentStackPointer(register_index) => {
                let parent = self.parent.ok_or(VmError::ExpectedParent { position })?;

                parent.open_register(*register_index, position)
            }
            Register::ParentConstantPointer(constant_index) => {
                let parent = self
                    .parent
                    .as_ref()
                    .ok_or(VmError::ExpectedParent { position })?;
                let constant = parent.get_constant(*constant_index, position)?;

                Ok(constant)
            }
            Register::Empty => Err(VmError::EmptyRegister {
                index: register_index,
                position,
            }),
        }
    }

    fn get_concrete_from_register(
        &'stack self,
        register_index: u8,
        position: Span,
    ) -> Result<ConcreteValue, VmError> {
        let register_index = register_index as usize;
        let register =
            self.stack
                .get(register_index)
                .ok_or_else(|| VmError::RegisterIndexOutOfBounds {
                    index: register_index,
                    position,
                })?;

        let value = match register {
            Register::Value(concrete_value) => concrete_value.clone(),
            Register::List(list) => {
                let items = list.into_iter().map(|value| (*value).clone()).collect();

                ConcreteValue::List(items)
            }
            Register::StackPointer(register_index) => {
                self.get_concrete_from_register(*register_index, position)?
            }
            Register::ConstantPointer(constant_pointer) => {
                self.get_constant(*constant_pointer, position)?.clone()
            }
            Register::ParentStackPointer(register_index) => {
                let parent = self.parent.ok_or(VmError::ExpectedParent { position })?;

                parent.get_concrete_from_register(*register_index, position)?
            }
            Register::ParentConstantPointer(constant_index) => {
                let parent = self
                    .parent
                    .as_ref()
                    .ok_or(VmError::ExpectedParent { position })?;

                parent.get_constant(*constant_index, position)?.clone()
            }
            Register::Empty => {
                return Err(VmError::EmptyRegister {
                    index: register_index,
                    position,
                })
            }
        };

        Ok(value)
    }

    /// DRY helper for handling JUMP instructions
    fn jump(&mut self, jump: Instruction) {
        let jump_distance = jump.b();
        let is_positive = jump.c_as_boolean();
        let new_ip = if is_positive {
            self.ip + jump_distance as usize
        } else {
            self.ip - jump_distance as usize - 1
        };
        self.ip = new_ip;
    }

    /// DRY helper to get a constant or register values
    fn get_argument(
        &'stack self,
        index: u8,
        is_constant: bool,
        position: Span,
    ) -> Result<Value<'stack>, VmError> {
        let argument = if is_constant {
            self.get_constant(index, position)?.as_reference_value()
        } else {
            self.open_register(index, position)?
        };

        Ok(argument)
    }

    /// DRY helper to get two arguments for binary operations
    fn get_arguments(
        &'stack self,
        instruction: Instruction,
        position: Span,
    ) -> Result<(Value<'stack>, Value<'stack>), VmError> {
        let left = self.get_argument(instruction.b(), instruction.b_is_constant(), position)?;
        let right = self.get_argument(instruction.c(), instruction.c_is_constant(), position)?;

        Ok((left, right))
    }

    fn set_register(
        &mut self,
        to_register: u8,
        register: Register<'stack>,
        position: Span,
    ) -> Result<(), VmError> {
        self.last_assigned_register = Some(to_register);

        let length = self.stack.len();
        let to_register = to_register as usize;

        if length == Self::STACK_LIMIT {
            return Err(VmError::StackOverflow { position });
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

    fn get_constant(&self, index: u8, position: Span) -> Result<&'chunk ConcreteValue, VmError> {
        self.chunk
            .get_constant(index)
            .map_err(|error| VmError::Chunk { error, position })
    }

    fn read(&mut self, position: Span) -> Result<&(Instruction, Span), VmError> {
        let ip = self.ip;

        self.ip += 1;

        self.get_instruction(ip, position)
    }

    fn get_instruction(
        &self,
        index: usize,
        position: Span,
    ) -> Result<&(Instruction, Span), VmError> {
        self.chunk
            .get_instruction(index)
            .map_err(|error| VmError::Chunk { error, position })
    }

    fn define_local(&mut self, local_index: u8, register_index: u8) -> Result<(), VmError> {
        log::debug!("Define local L{}", local_index);

        self.local_definitions.insert(local_index, register_index);

        Ok(())
    }
}

#[derive(Debug, Eq, PartialEq)]
enum Register<'stack> {
    Empty,
    Value(ConcreteValue),
    List(Vec<&'stack ConcreteValue>),
    StackPointer(u8),
    ConstantPointer(u8),
    ParentStackPointer(u8),
    ParentConstantPointer(u8),
}

impl<'stack> Display for Register<'stack> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Self::Empty => write!(f, "empty"),
            Self::Value(value) => write!(f, "{}", value),
            Self::List(values) => {
                write!(f, "[")?;

                for (index, value) in values.iter().enumerate() {
                    if index > 0 {
                        write!(f, ", ")?;
                    }

                    write!(f, "{}", value)?;
                }

                write!(f, "]")
            }
            Self::StackPointer(index) => write!(f, "R{}", index),
            Self::ConstantPointer(index) => write!(f, "C{}", index),
            Self::ParentStackPointer(index) => write!(f, "PR{}", index),
            Self::ParentConstantPointer(index) => write!(f, "PC{}", index),
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
        found: ConcreteValue,
        position: Span,
    },
    ExpectedFunction {
        found: ConcreteValue,
        position: Span,
    },
    ExpectedParent {
        position: Span,
    },

    // Wrappers for foreign errors
    Chunk {
        error: ChunkError,
        position: Span,
    },
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
            Self::Chunk { .. } => "Chunk error",
            Self::EmptyRegister { .. } => "Empty register",
            Self::ExpectedBoolean { .. } => "Expected boolean",
            Self::ExpectedFunction { .. } => "Expected function",
            Self::ExpectedParent { .. } => "Expected parent",
            Self::NativeFunction(error) => error.description(),
            Self::RegisterIndexOutOfBounds { .. } => "Register index out of bounds",
            Self::StackOverflow { .. } => "Stack overflow",
            Self::StackUnderflow { .. } => "Stack underflow",
            Self::UndefinedLocal { .. } => "Undefined local",
            Self::Value { .. } => "Value error",
        }
    }

    fn details(&self) -> Option<String> {
        match self {
            Self::Chunk { error, .. } => Some(error.to_string()),
            Self::EmptyRegister { index, .. } => Some(format!("Register R{index} is empty")),
            Self::ExpectedFunction { found, .. } => Some(format!("{found} is not a function")),

            Self::RegisterIndexOutOfBounds { index, .. } => {
                Some(format!("Register {index} does not exist"))
            }
            Self::NativeFunction(error) => error.details(),
            Self::Value { error, .. } => Some(error.to_string()),
            _ => None,
        }
    }

    fn position(&self) -> Span {
        match self {
            Self::Chunk { position, .. } => *position,
            Self::EmptyRegister { position, .. } => *position,
            Self::ExpectedBoolean { position, .. } => *position,
            Self::ExpectedFunction { position, .. } => *position,
            Self::ExpectedParent { position } => *position,
            Self::NativeFunction(error) => error.position(),
            Self::RegisterIndexOutOfBounds { position, .. } => *position,
            Self::StackOverflow { position } => *position,
            Self::StackUnderflow { position } => *position,
            Self::UndefinedLocal { position, .. } => *position,
            Self::Value { position, .. } => *position,
        }
    }
}
