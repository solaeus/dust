//! Virtual machine and errors
use std::{
    cmp::Ordering,
    collections::HashMap,
    fmt::{self, Display, Formatter},
};

use crate::{
    compile, value::Value, AnnotatedError, Chunk, ChunkError, DustError, Instruction,
    NativeFunction, NativeFunctionError, Operation, Span, Type, ValueError,
};

pub fn run(source: &str) -> Result<Option<Value>, DustError> {
    let chunk = compile(source)?;
    let has_return_value = *chunk.r#type().return_type != Type::None;
    let mut vm = Vm::new(&chunk, None);

    vm.run()
        .map_err(|error| DustError::Runtime { error, source })?;

    if has_return_value {
        vm.take_top_of_stack_as_value()
            .map(Some)
            .map_err(|error| DustError::Runtime { error, source })
    } else {
        Ok(None)
    }
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
#[derive(Debug, PartialEq)]
pub struct Vm<'a> {
    chunk: &'a Chunk,
    stack: Vec<Register>,
    parent: Option<&'a Vm<'a>>,

    ip: usize,
    local_definitions: HashMap<u8, u8>,
    last_assigned_register: Option<u8>,
    current_position: Span,
}

impl<'a> Vm<'a> {
    const STACK_LIMIT: usize = u16::MAX as usize;

    pub fn new(chunk: &'a Chunk, parent: Option<&'a Vm<'a>>) -> Self {
        Self {
            chunk,
            stack: Vec::new(),
            parent,
            ip: 0,
            local_definitions: HashMap::new(),
            last_assigned_register: None,
            current_position: Span(0, 0),
        }
    }

    pub fn current_position(&self) -> Span {
        self.current_position
    }

    pub fn run(&mut self) -> Result<(), VmError> {
        while let Ok(instruction) = self.read() {
            log::info!(
                "{} | {} | {} | {}",
                self.ip - 1,
                self.current_position,
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
                    let register = Register::Pointer(Pointer::Stack(from_register));

                    if from_register_has_value {
                        self.set_register(to_register, register)?;
                    }
                }
                Operation::Close => {
                    let from_register = instruction.b();
                    let to_register = instruction.c();

                    if self.stack.len() < to_register as usize {
                        return Err(VmError::StackUnderflow {
                            position: self.current_position,
                        });
                    }

                    for register_index in from_register..to_register {
                        self.stack[register_index as usize] = Register::Empty;
                    }
                }
                Operation::LoadBoolean => {
                    let to_register = instruction.a();
                    let boolean = instruction.b_as_boolean();
                    let jump = instruction.c_as_boolean();
                    let boolean = Value::boolean(boolean);

                    self.set_register(to_register, Register::Value(boolean))?;

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
                        Register::Pointer(Pointer::Constant(from_constant)),
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
                        let value = self.open_register(register_index)?;

                        list.push(value);
                    }

                    // self.set_register(to_register, Register::List(list))?;

                    todo!()
                }
                Operation::LoadSelf => {
                    let to_register = instruction.a();

                    // self.set_register(to_register, Register::Value(function))?;

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
                            position: self.current_position,
                        },
                    )?;
                    let register = Register::Pointer(Pointer::Stack(local_register));

                    self.set_register(to_register, register)?;
                }
                Operation::SetLocal => {
                    let from_register = instruction.a();
                    let to_local = instruction.b();
                    let local_register = self.local_definitions.get(&to_local).copied().ok_or(
                        VmError::UndefinedLocal {
                            local_index: to_local,
                            position: self.current_position,
                        },
                    )?;
                    let register = Register::Pointer(Pointer::Stack(from_register));

                    self.set_register(local_register, register)?;
                }
                Operation::Add => {
                    let to_register = instruction.a();
                    let (left, right) = self.get_arguments(instruction)?;
                    let sum = left.add(right).map_err(|error| VmError::Value {
                        error,
                        position: self.current_position,
                    })?;

                    self.set_register(to_register, Register::Value(sum))?;
                }
                Operation::Subtract => {
                    let to_register = instruction.a();
                    let (left, right) = self.get_arguments(instruction)?;
                    let difference = left.subtract(right).map_err(|error| VmError::Value {
                        error,
                        position: self.current_position,
                    })?;

                    self.set_register(to_register, Register::Value(difference))?;
                }
                Operation::Multiply => {
                    let to_register = instruction.a();
                    let (left, right) = self.get_arguments(instruction)?;
                    let product = left.multiply(right).map_err(|error| VmError::Value {
                        error,
                        position: self.current_position,
                    })?;

                    self.set_register(to_register, Register::Value(product))?;
                }
                Operation::Divide => {
                    let to_register = instruction.a();
                    let (left, right) = self.get_arguments(instruction)?;
                    let quotient = left.divide(right).map_err(|error| VmError::Value {
                        error,
                        position: self.current_position,
                    })?;

                    self.set_register(to_register, Register::Value(quotient))?;
                }
                Operation::Modulo => {
                    let to_register = instruction.a();
                    let (left, right) = self.get_arguments(instruction)?;
                    let remainder = left.modulo(right).map_err(|error| VmError::Value {
                        error,
                        position: self.current_position,
                    })?;

                    self.set_register(to_register, Register::Value(remainder))?;
                }
                Operation::Test => {
                    let register = instruction.a();
                    let test_value = instruction.c_as_boolean();
                    let value = self.open_register(register)?;
                    let boolean = if let Value::Boolean(boolean) = value {
                        *boolean
                    } else {
                        return Err(VmError::ExpectedBoolean {
                            found: value.clone(),
                            position: self.current_position,
                        });
                    };

                    if boolean != test_value {
                        self.ip += 1;
                    }
                }
                Operation::TestSet => todo!(),
                Operation::Equal => {
                    debug_assert_eq!(
                        self.get_instruction(self.ip)?.0.operation(),
                        Operation::Jump
                    );

                    let compare_to = instruction.a_as_boolean();
                    let (left, right) = self.get_arguments(instruction)?;
                    let equal_result = left.equal(right).map_err(|error| VmError::Value {
                        error,
                        position: self.current_position,
                    })?;
                    let is_equal = if let Value::Boolean(boolean) = equal_result {
                        boolean
                    } else {
                        return Err(VmError::ExpectedBoolean {
                            found: equal_result.clone(),
                            position: self.current_position,
                        });
                    };

                    if is_equal == compare_to {
                        self.ip += 1;
                    } else {
                        let jump = self.get_instruction(self.ip)?.0;

                        self.jump(jump);
                    }
                }
                Operation::Less => {
                    debug_assert_eq!(
                        self.get_instruction(self.ip)?.0.operation(),
                        Operation::Jump
                    );

                    let compare_to = instruction.a_as_boolean();
                    let (left, right) = self.get_arguments(instruction)?;
                    let less_result = left.less_than(right).map_err(|error| VmError::Value {
                        error,
                        position: self.current_position,
                    })?;
                    let is_less_than = if let Value::Boolean(boolean) = less_result {
                        boolean
                    } else {
                        return Err(VmError::ExpectedBoolean {
                            found: less_result.clone(),
                            position: self.current_position,
                        });
                    };

                    if is_less_than == compare_to {
                        self.ip += 1;
                    } else {
                        let jump = self.get_instruction(self.ip)?.0;

                        self.jump(jump);
                    }
                }
                Operation::LessEqual => {
                    debug_assert_eq!(
                        self.get_instruction(self.ip)?.0.operation(),
                        Operation::Jump
                    );

                    let compare_to = instruction.a_as_boolean();
                    let (left, right) = self.get_arguments(instruction)?;
                    let less_or_equal_result =
                        left.less_than_or_equal(right)
                            .map_err(|error| VmError::Value {
                                error,
                                position: self.current_position,
                            })?;
                    let is_less_than_or_equal =
                        if let Value::Boolean(boolean) = less_or_equal_result {
                            boolean
                        } else {
                            return Err(VmError::ExpectedBoolean {
                                found: less_or_equal_result.clone(),
                                position: self.current_position,
                            });
                        };

                    if is_less_than_or_equal == compare_to {
                        self.ip += 1;
                    } else {
                        let jump = self.get_instruction(self.ip)?.0;

                        self.jump(jump);
                    }
                }
                Operation::Negate => {
                    let value = self.get_argument(instruction.b(), instruction.b_is_constant())?;
                    let negated = value.negate().map_err(|error| VmError::Value {
                        error,
                        position: self.current_position,
                    })?;

                    self.set_register(instruction.a(), Register::Value(negated))?;
                }
                Operation::Not => {
                    let value = self.get_argument(instruction.b(), instruction.b_is_constant())?;
                    let not = value.not().map_err(|error| VmError::Value {
                        error,
                        position: self.current_position,
                    })?;

                    self.set_register(instruction.a(), Register::Value(not))?;
                }
                Operation::Jump => self.jump(instruction),
                Operation::Call => {
                    let to_register = instruction.a();
                    let function_register = instruction.b();
                    let argument_count = instruction.c();
                    let value = self.open_register(function_register)?;
                    let chunk = if let Value::Function(chunk) = value {
                        chunk
                    } else {
                        return Err(VmError::ExpectedFunction {
                            found: value.clone(),
                            position: self.current_position,
                        });
                    };
                    let has_return_value = *chunk.r#type().return_type != Type::None;
                    let mut function_vm = Vm::new(chunk, Some(self));
                    let first_argument_index = function_register + 1;
                    let last_argument_index = first_argument_index + argument_count;

                    for argument_index in first_argument_index..last_argument_index {
                        let top_of_stack = function_vm.stack.len() as u8;

                        function_vm.set_register(
                            top_of_stack,
                            Register::Pointer(Pointer::ParentStack(argument_index)),
                        )?
                    }

                    function_vm.run()?;

                    if has_return_value {
                        let top_of_stack = function_vm.stack.len() as u8 - 1;

                        self.set_register(
                            to_register,
                            Register::Pointer(Pointer::ParentStack(top_of_stack)),
                        )?;
                    }
                }
                Operation::CallNative => {
                    let native_function = NativeFunction::from(instruction.b());
                    let return_value = native_function.call(self, instruction)?;

                    if let Some(concrete_value) = return_value {
                        let to_register = instruction.a();

                        self.set_register(to_register, Register::Value(concrete_value))?;
                    }
                }
                Operation::Return => {
                    let should_return_value = instruction.b_as_boolean();

                    if !should_return_value {
                        return Ok(());
                    }

                    return if let Some(register_index) = self.last_assigned_register {
                        let top_of_stack = self.stack.len() as u8 - 1;

                        if register_index != top_of_stack {
                            self.stack
                                .push(Register::Pointer(Pointer::Stack(register_index)));
                        }

                        Ok(())
                    } else {
                        Err(VmError::StackUnderflow {
                            position: self.current_position,
                        })
                    };
                }
            }
        }

        Ok(())
    }

    fn resolve_pointer(&self, pointer: Pointer) -> Result<&Value, VmError> {
        match pointer {
            Pointer::Stack(register_index) => self.open_register(register_index),
            Pointer::Constant(constant_index) => self.get_constant(constant_index),
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

                parent.get_constant(constant_index)
            }
        }
    }

    pub(crate) fn open_register(&self, register_index: u8) -> Result<&Value, VmError> {
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
            Register::Value(value) => Ok(value),
            Register::Pointer(pointer) => self.resolve_pointer(*pointer),
            Register::Empty => Err(VmError::EmptyRegister {
                index: register_index,
                position: self.current_position,
            }),
        }
    }

    fn take_top_of_stack_as_value(&mut self) -> Result<Value, VmError> {
        let top_of_stack = self.stack.pop().ok_or(VmError::StackUnderflow {
            position: self.current_position,
        })?;

        match top_of_stack {
            Register::Value(value) => Ok(value),
            _ => Err(VmError::ExpectedValue {
                found: top_of_stack,
                position: self.current_position,
            }),
        }
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
    fn get_argument(&self, index: u8, is_constant: bool) -> Result<&Value, VmError> {
        let argument = if is_constant {
            self.get_constant(index)?
        } else {
            self.open_register(index)?
        };

        Ok(argument)
    }

    /// DRY helper to get two arguments for binary operations
    fn get_arguments(&self, instruction: Instruction) -> Result<(&Value, &Value), VmError> {
        let left = self.get_argument(instruction.b(), instruction.b_is_constant())?;
        let right = self.get_argument(instruction.c(), instruction.c_is_constant())?;

        Ok((left, right))
    }

    fn set_register(&mut self, to_register: u8, register: Register) -> Result<(), VmError> {
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

    fn get_constant(&self, index: u8) -> Result<&Value, VmError> {
        self.chunk
            .get_constant(index)
            .map_err(|error| VmError::Chunk {
                error,
                position: self.current_position,
            })
    }

    fn read(&mut self) -> Result<Instruction, VmError> {
        let (instruction, position) = *self.get_instruction(self.ip)?;

        self.ip += 1;
        self.current_position = position;

        Ok(instruction)
    }

    fn get_instruction(&self, index: usize) -> Result<&(Instruction, Span), VmError> {
        self.chunk
            .get_instruction(index)
            .map_err(|error| VmError::Chunk {
                error,
                position: self.current_position,
            })
    }

    fn define_local(&mut self, local_index: u8, register_index: u8) -> Result<(), VmError> {
        log::debug!("Define local L{}", local_index);

        self.local_definitions.insert(local_index, register_index);

        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq)]
enum Register {
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
    StackOverflow { position: Span },
    StackUnderflow { position: Span },

    // Register errors
    EmptyRegister { index: usize, position: Span },
    ExpectedValue { found: Register, position: Span },
    RegisterIndexOutOfBounds { index: usize, position: Span },

    // Local errors
    UndefinedLocal { local_index: u8, position: Span },

    // Execution errors
    ExpectedBoolean { found: Value, position: Span },
    ExpectedFunction { found: Value, position: Span },
    ExpectedParent { position: Span },

    // Wrappers for foreign errors
    Chunk { error: ChunkError, position: Span },
    NativeFunction(NativeFunctionError),
    Value { error: ValueError, position: Span },
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
            Self::ExpectedValue { .. } => "Expected value",
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
            Self::ExpectedValue { position, .. } => *position,
            Self::NativeFunction(error) => error.position(),
            Self::RegisterIndexOutOfBounds { position, .. } => *position,
            Self::StackOverflow { position } => *position,
            Self::StackUnderflow { position } => *position,
            Self::UndefinedLocal { position, .. } => *position,
            Self::Value { position, .. } => *position,
        }
    }
}
