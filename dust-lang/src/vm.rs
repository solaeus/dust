use log::debug;

use crate::{
    parse, AnnotatedError, Chunk, ChunkError, DustError, Identifier, Instruction, Local, Operation,
    Span, Value, ValueError,
};

pub fn run(source: &str) -> Result<Option<Value>, DustError> {
    let chunk = parse(source)?;

    let mut vm = Vm::new(chunk);

    vm.run()
        .map_err(|error| DustError::Runtime { error, source })
}

#[derive(Debug, Eq, PartialEq)]
pub struct Vm {
    chunk: Chunk,
    ip: usize,
    register_stack: Vec<Option<Value>>,
}

impl Vm {
    const STACK_LIMIT: usize = u16::MAX as usize;

    pub fn new(chunk: Chunk) -> Self {
        Self {
            chunk,
            ip: 0,
            register_stack: Vec::new(),
        }
    }

    pub fn take_chunk(self) -> Chunk {
        self.chunk
    }

    pub fn run(&mut self) -> Result<Option<Value>, VmError> {
        // DRY helper to get constant or register values for binary operations
        fn get_arguments(
            vm: &mut Vm,
            instruction: Instruction,
            position: Span,
        ) -> Result<(&Value, &Value), VmError> {
            let left = if instruction.first_argument_is_constant() {
                vm.chunk
                    .get_constant(instruction.first_argument(), position)?
            } else {
                vm.get(instruction.first_argument(), position)?
            };
            let right = if instruction.second_argument_is_constant() {
                vm.chunk
                    .get_constant(instruction.second_argument(), position)?
            } else {
                vm.get(instruction.second_argument(), position)?
            };

            Ok((left, right))
        }

        while let Ok((instruction, position)) = self.read(Span(0, 0)).copied() {
            log::trace!(
                "Running IP {} {} at {position}",
                self.ip - 1,
                instruction.operation()
            );

            match instruction.operation() {
                Operation::Move => {
                    let from = instruction.first_argument();
                    let to = instruction.destination();
                    let value = self.take(from, position)?;

                    self.insert(value, to, position)?;
                }
                Operation::Close => {
                    let from = instruction.first_argument();
                    let to = instruction.second_argument();

                    for register_index in from..to {
                        self.register_stack[register_index as usize] = None;
                    }
                }
                Operation::LoadBoolean => {
                    let to_register = instruction.destination();
                    let boolean = instruction.first_argument_as_boolean();
                    let skip = instruction.second_argument_as_boolean();
                    let value = Value::boolean(boolean);

                    self.insert(value, to_register, position)?;

                    if skip {
                        self.ip += 1;
                    }
                }
                Operation::LoadConstant => {
                    let to_register = instruction.destination();
                    let from_constant = instruction.first_argument();
                    let value = self.chunk.take_constant(from_constant, position)?;

                    self.insert(value, to_register, position)?;
                }
                Operation::LoadList => {
                    let to_register = instruction.destination();
                    let length = instruction.first_argument();
                    let first_register = to_register - length - 1;
                    let last_register = to_register - 1;

                    let mut list = Vec::with_capacity(length as usize);

                    for register_index in first_register..=last_register {
                        let value = match self.clone(register_index, position) {
                            Ok(value) => value,
                            Err(VmError::EmptyRegister { .. }) => continue,
                            Err(error) => return Err(error),
                        };

                        list.push(value);
                    }

                    self.insert(Value::list(list), to_register, position)?;
                }
                Operation::DefineLocal => {
                    let from_register = instruction.destination();
                    let to_local = instruction.first_argument();

                    self.chunk.define_local(to_local, from_register, position)?;
                }
                Operation::GetLocal => {
                    let register_index = instruction.destination();
                    let local_index = instruction.first_argument();
                    let local = self.chunk.get_local(local_index, position)?.clone();
                    let value = self.clone_as_variable(local, position)?;

                    self.insert(value, register_index, position)?;
                }
                Operation::SetLocal => {
                    let register_index = instruction.destination();
                    let local_index = instruction.first_argument();
                    let local = self.chunk.get_local(local_index, position)?.clone();
                    let value = self.clone_as_variable(local, position)?;
                    let new_value = if instruction.first_argument_is_constant() {
                        self.chunk.take_constant(register_index, position)?
                    } else {
                        self.clone(register_index, position)?
                    };

                    value
                        .mutate(new_value)
                        .map_err(|error| VmError::Value { error, position })?;

                    self.insert(value, register_index, position)?;
                }
                Operation::Add => {
                    let (left, right) = get_arguments(self, instruction, position)?;
                    let sum = left
                        .add(right)
                        .map_err(|error| VmError::Value { error, position })?;

                    self.insert(sum, instruction.destination(), position)?;
                }
                Operation::Subtract => {
                    let (left, right) = get_arguments(self, instruction, position)?;
                    let difference = left
                        .subtract(right)
                        .map_err(|error| VmError::Value { error, position })?;

                    self.insert(difference, instruction.destination(), position)?;
                }
                Operation::Multiply => {
                    let (left, right) = get_arguments(self, instruction, position)?;
                    let product = left
                        .multiply(right)
                        .map_err(|error| VmError::Value { error, position })?;

                    self.insert(product, instruction.destination(), position)?;
                }
                Operation::Divide => {
                    let (left, right) = get_arguments(self, instruction, position)?;
                    let quotient = left
                        .divide(right)
                        .map_err(|error| VmError::Value { error, position })?;

                    self.insert(quotient, instruction.destination(), position)?;
                }
                Operation::Modulo => {
                    let (left, right) = get_arguments(self, instruction, position)?;
                    let remainder = left
                        .modulo(right)
                        .map_err(|error| VmError::Value { error, position })?;

                    self.insert(remainder, instruction.destination(), position)?;
                }
                Operation::Test => {
                    let register = instruction.destination();
                    let test_value = instruction.second_argument_as_boolean();
                    let value = self.get(register, position)?;
                    let boolean = value.as_boolean().ok_or_else(|| VmError::ExpectedBoolean {
                        found: value.clone(),
                        position,
                    })?;

                    if boolean == test_value {
                        self.ip += 1;
                    }
                }
                Operation::TestSet => {
                    let to_register = instruction.destination();
                    let argument = instruction.first_argument();
                    let test_value = instruction.second_argument_as_boolean();
                    let value = self.clone(argument, position)?;
                    let boolean = value.as_boolean().ok_or_else(|| VmError::ExpectedBoolean {
                        found: value.clone(),
                        position,
                    })?;

                    if boolean == test_value {
                        self.insert(value, to_register, position)?;
                    } else {
                        self.ip += 1;
                    }
                }
                Operation::Equal => {
                    let (jump, _) = *self.chunk.get_instruction(self.ip, position)?;

                    debug_assert_eq!(jump.operation(), Operation::Jump);

                    let (left, right) = get_arguments(self, instruction, position)?;
                    let boolean = left
                        .equal(right)
                        .map_err(|error| VmError::Value { error, position })?
                        .as_boolean()
                        .ok_or_else(|| VmError::ExpectedBoolean {
                            found: left.clone(),
                            position,
                        })?;
                    let compare_to = instruction.destination_as_boolean();

                    if boolean == compare_to {
                        self.ip += 1;
                    } else {
                        let jump_distance = jump.destination();
                        let is_positive = jump.first_argument_as_boolean();
                        let new_ip = if is_positive {
                            self.ip + jump_distance as usize
                        } else {
                            self.ip - jump_distance as usize
                        };

                        self.ip = new_ip;
                    }
                }
                Operation::Less => {
                    let (jump, _) = *self.chunk.get_instruction(self.ip, position)?;

                    assert_eq!(jump.operation(), Operation::Jump);

                    let (left, right) = get_arguments(self, instruction, position)?;
                    let less = left
                        .less_than(right)
                        .map_err(|error| VmError::Value { error, position })?;
                    let boolean = left
                        .equal(right)
                        .map_err(|error| VmError::Value { error, position })?
                        .as_boolean()
                        .ok_or_else(|| VmError::ExpectedBoolean {
                            found: less,
                            position,
                        })?;
                    let compare_to = instruction.destination_as_boolean();

                    if boolean == compare_to {
                        self.ip += 1;
                    } else {
                        let jump_distance = jump.destination();
                        let is_positive = jump.first_argument_as_boolean();
                        let new_ip = if is_positive {
                            self.ip + jump_distance as usize
                        } else {
                            self.ip - jump_distance as usize
                        };

                        self.ip = new_ip;
                    }
                }
                Operation::LessEqual => {
                    let (jump, _) = *self.read(position)?;
                    let (left, right) = get_arguments(self, instruction, position)?;
                    let less_or_equal = left
                        .less_than_or_equal(right)
                        .map_err(|error| VmError::Value { error, position })?;
                    let boolean = left
                        .equal(right)
                        .map_err(|error| VmError::Value { error, position })?
                        .as_boolean()
                        .ok_or_else(|| VmError::ExpectedBoolean {
                            found: less_or_equal,
                            position,
                        })?;
                    let compare_to = instruction.destination_as_boolean();

                    if boolean == compare_to {
                        self.ip += 1;
                    } else {
                        let jump_distance = jump.destination();
                        let is_positive = jump.first_argument_as_boolean();
                        let new_ip = if is_positive {
                            self.ip + jump_distance as usize
                        } else {
                            self.ip - jump_distance as usize
                        };

                        self.ip = new_ip;
                    }
                }
                Operation::Negate => {
                    let value = if instruction.first_argument_is_constant() {
                        self.chunk
                            .get_constant(instruction.first_argument(), position)?
                    } else {
                        self.get(instruction.first_argument(), position)?
                    };
                    let negated = value
                        .negate()
                        .map_err(|error| VmError::Value { error, position })?;

                    self.insert(negated, instruction.destination(), position)?;
                }
                Operation::Not => {
                    let value = if instruction.first_argument_is_constant() {
                        self.chunk
                            .get_constant(instruction.first_argument(), position)?
                    } else {
                        self.get(instruction.first_argument(), position)?
                    };
                    let not = value
                        .not()
                        .map_err(|error| VmError::Value { error, position })?;

                    self.insert(not, instruction.destination(), position)?;
                }
                Operation::Jump => {
                    let offset = instruction.first_argument();
                    let is_positive = instruction.second_argument_as_boolean();
                    let new_ip = if is_positive {
                        self.ip + offset as usize
                    } else {
                        self.ip - offset as usize
                    };

                    self.ip = new_ip;
                }
                Operation::Return => {
                    let start_register = instruction.destination();
                    let end_register = instruction.first_argument();
                    let return_value_count = end_register - start_register;

                    if return_value_count == 1 {
                        return Ok(Some(self.take(start_register, position)?));
                    }
                }
                Operation::End => {
                    let returns_value = instruction.destination_as_boolean();

                    if returns_value {
                        return Ok(Some(self.pop(position)?));
                    }
                }
            }
        }

        Ok(None)
    }

    fn insert(&mut self, value: Value, index: u8, position: Span) -> Result<(), VmError> {
        if self.register_stack.len() == Self::STACK_LIMIT {
            Err(VmError::StackOverflow { position })
        } else {
            let index = index as usize;

            while index >= self.register_stack.len() {
                self.register_stack.push(None);
            }

            self.register_stack[index] = Some(value);

            Ok(())
        }
    }

    fn take(&mut self, index: u8, position: Span) -> Result<Value, VmError> {
        let index = index as usize;

        if let Some(register) = self.register_stack.get_mut(index) {
            let value = register
                .take()
                .ok_or_else(|| VmError::EmptyRegister { index, position })?;

            Ok(value)
        } else {
            Err(VmError::RegisterIndexOutOfBounds { position })
        }
    }

    fn get(&self, index: u8, position: Span) -> Result<&Value, VmError> {
        let index = index as usize;

        if let Some(register) = self.register_stack.get(index) {
            let value = register
                .as_ref()
                .ok_or_else(|| VmError::EmptyRegister { index, position })?;

            Ok(value)
        } else {
            Err(VmError::RegisterIndexOutOfBounds { position })
        }
    }

    fn clone(&mut self, index: u8, position: Span) -> Result<Value, VmError> {
        let index = index as usize;

        if let Some(register) = self.register_stack.get_mut(index) {
            let cloneable = if let Some(value) = register.take() {
                if value.is_raw() {
                    value.into_reference()
                } else {
                    value
                }
            } else {
                return Err(VmError::EmptyRegister { index, position });
            };

            *register = Some(cloneable.clone());

            Ok(cloneable)
        } else {
            Err(VmError::RegisterIndexOutOfBounds { position })
        }
    }

    fn clone_mutable(&mut self, index: u8, position: Span) -> Result<Value, VmError> {
        let index = index as usize;

        if let Some(register) = self.register_stack.get_mut(index) {
            let cloneable = if let Some(value) = register.take() {
                value.into_mutable()
            } else {
                return Err(VmError::EmptyRegister { index, position });
            };

            *register = Some(cloneable.clone());

            Ok(cloneable)
        } else {
            Err(VmError::RegisterIndexOutOfBounds { position })
        }
    }

    fn clone_as_variable(&mut self, local: Local, position: Span) -> Result<Value, VmError> {
        let index = if let Some(index) = local.register_index {
            index
        } else {
            return Err(VmError::UndefinedVariable {
                identifier: local.identifier,
                position,
            });
        };
        let clone_result = if local.mutable {
            self.clone_mutable(index, position)
        } else {
            self.clone(index, position)
        };

        match clone_result {
            Err(VmError::EmptyRegister { .. }) => Err(VmError::UndefinedVariable {
                identifier: local.identifier,
                position,
            }),
            _ => clone_result,
        }
    }

    fn pop(&mut self, position: Span) -> Result<Value, VmError> {
        if let Some(register) = self.register_stack.pop() {
            let value = register.ok_or(VmError::EmptyRegister {
                index: self.register_stack.len().saturating_sub(1),
                position,
            })?;

            Ok(value)
        } else {
            Err(VmError::StackUnderflow { position })
        }
    }

    fn read(&mut self, position: Span) -> Result<&(Instruction, Span), VmError> {
        let current = self.chunk.get_instruction(self.ip, position)?;

        self.ip += 1;

        Ok(current)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum VmError {
    EmptyRegister {
        index: usize,
        position: Span,
    },
    ExpectedBoolean {
        found: Value,
        position: Span,
    },
    RegisterIndexOutOfBounds {
        position: Span,
    },
    InvalidInstruction {
        instruction: Instruction,
        position: Span,
    },
    StackOverflow {
        position: Span,
    },
    StackUnderflow {
        position: Span,
    },
    UndefinedVariable {
        identifier: Identifier,
        position: Span,
    },

    // Wrappers for foreign errors
    Chunk(ChunkError),
    Value {
        error: ValueError,
        position: Span,
    },
}

impl From<ChunkError> for VmError {
    fn from(error: ChunkError) -> Self {
        Self::Chunk(error)
    }
}

impl AnnotatedError for VmError {
    fn title() -> &'static str {
        "Runtime Error"
    }

    fn description(&self) -> &'static str {
        match self {
            Self::EmptyRegister { .. } => "Empty register",
            Self::ExpectedBoolean { .. } => "Expected boolean",
            Self::RegisterIndexOutOfBounds { .. } => "Register index out of bounds",
            Self::InvalidInstruction { .. } => "Invalid instruction",
            Self::StackOverflow { .. } => "Stack overflow",
            Self::StackUnderflow { .. } => "Stack underflow",
            Self::UndefinedVariable { .. } => "Undefined variable",
            Self::Value { .. } => "Value error",
            Self::Chunk(error) => error.description(),
        }
    }

    fn details(&self) -> Option<String> {
        match self {
            Self::EmptyRegister { index, .. } => Some(format!("Register {index} is empty")),
            Self::UndefinedVariable { identifier, .. } => {
                Some(format!("{identifier} is not in scope"))
            }
            Self::Chunk(error) => error.details(),
            Self::Value { error, .. } => Some(error.to_string()),
            _ => None,
        }
    }

    fn position(&self) -> Span {
        match self {
            Self::EmptyRegister { position, .. } => *position,
            Self::ExpectedBoolean { position, .. } => *position,
            Self::RegisterIndexOutOfBounds { position } => *position,
            Self::InvalidInstruction { position, .. } => *position,
            Self::StackUnderflow { position } => *position,
            Self::StackOverflow { position } => *position,
            Self::UndefinedVariable { position, .. } => *position,
            Self::Chunk(error) => error.position(),
            Self::Value { position, .. } => *position,
        }
    }
}
