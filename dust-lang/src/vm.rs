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
            let left = if instruction.b_is_constant() {
                vm.chunk.get_constant(instruction.b(), position)?
            } else {
                vm.get(instruction.b(), position)?
            };
            let right = if instruction.c_is_constant() {
                vm.chunk.get_constant(instruction.c(), position)?
            } else {
                vm.get(instruction.c(), position)?
            };

            Ok((left, right))
        }

        while let Ok((instruction, position)) = self.read(Span(0, 0)).copied() {
            let ip = self.ip - 1;
            let operation = instruction.operation();
            let position_display = position.to_string();

            log::info!("{ip:^3} | {position_display:^10} | {operation}");

            match instruction.operation() {
                Operation::Move => {
                    let from = instruction.b();
                    let to = instruction.a();
                    let value = self.take(from, position)?;

                    self.insert(value, to, position)?;
                }
                Operation::Close => {
                    let from = instruction.b();
                    let to = instruction.c();

                    for register_index in from..to {
                        self.register_stack[register_index as usize] = None;
                    }
                }
                Operation::LoadBoolean => {
                    let to_register = instruction.a();
                    let boolean = instruction.b_as_boolean();
                    let skip = instruction.c_as_boolean();
                    let value = Value::boolean(boolean);

                    self.insert(value, to_register, position)?;

                    if skip {
                        self.ip += 1;
                    }
                }
                Operation::LoadConstant => {
                    let to_register = instruction.a();
                    let from_constant = instruction.b();
                    let jump = instruction.c_as_boolean();
                    let value = self.chunk.take_constant(from_constant, position)?;

                    self.insert(value, to_register, position)?;

                    if jump {
                        self.ip += 1;
                    }
                }
                Operation::LoadList => {
                    let to_register = instruction.a();
                    let first_register = instruction.b();
                    let last_register = instruction.c();
                    let length = last_register - first_register + 1;
                    let mut list = Vec::with_capacity(length as usize);

                    for register_index in first_register..=last_register {
                        let value = match self.take(register_index, position) {
                            Ok(value) => value,
                            Err(VmError::EmptyRegister { .. }) => continue,
                            Err(error) => return Err(error),
                        };

                        list.push(value);
                    }

                    self.insert(Value::list(list), to_register, position)?;
                }
                Operation::DefineLocal => {
                    let from_register = instruction.a();
                    let to_local = instruction.b();

                    self.chunk.define_local(to_local, from_register, position)?;
                }
                Operation::GetLocal => {
                    let register_index = instruction.a();
                    let local_index = instruction.b();
                    let local = self.chunk.get_local(local_index, position)?;
                    let value = if let Some(index) = local.register_index {
                        self.take(index, position)?
                    } else {
                        return Err(VmError::UndefinedVariable {
                            identifier: local.identifier.clone(),
                            position,
                        });
                    };

                    self.insert(value, register_index, position)?;
                }
                Operation::SetLocal => {
                    let register_index = instruction.a();
                    let local_index = instruction.b();
                    let local = self.chunk.get_local(local_index, position)?;
                    let value = if let Some(index) = local.register_index {
                        self.take(index, position)?
                    } else {
                        return Err(VmError::UndefinedVariable {
                            identifier: local.identifier.clone(),
                            position,
                        });
                    };

                    let new_value = if instruction.b_is_constant() {
                        self.chunk.take_constant(register_index, position)?
                    } else {
                        self.take(register_index, position)?
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

                    self.insert(sum, instruction.a(), position)?;
                }
                Operation::Subtract => {
                    let (left, right) = get_arguments(self, instruction, position)?;
                    let difference = left
                        .subtract(right)
                        .map_err(|error| VmError::Value { error, position })?;

                    self.insert(difference, instruction.a(), position)?;
                }
                Operation::Multiply => {
                    let (left, right) = get_arguments(self, instruction, position)?;

                    let product = left
                        .multiply(right)
                        .map_err(|error| VmError::Value { error, position })?;

                    self.insert(product, instruction.a(), position)?;
                }
                Operation::Divide => {
                    let (left, right) = get_arguments(self, instruction, position)?;
                    let quotient = left
                        .divide(right)
                        .map_err(|error| VmError::Value { error, position })?;

                    self.insert(quotient, instruction.a(), position)?;
                }
                Operation::Modulo => {
                    let (left, right) = get_arguments(self, instruction, position)?;
                    let remainder = left
                        .modulo(right)
                        .map_err(|error| VmError::Value { error, position })?;

                    self.insert(remainder, instruction.a(), position)?;
                }
                Operation::Test => {
                    let register = instruction.a();
                    let test_value = instruction.c_as_boolean();
                    let value = self.get(register, position)?;
                    let boolean = value.as_boolean().ok_or_else(|| VmError::ExpectedBoolean {
                        found: value.clone(),
                        position,
                    })?;

                    if boolean != test_value {
                        self.ip += 1;
                    }
                }
                Operation::TestSet => {
                    let to_register = instruction.a();
                    let argument = instruction.b();
                    let test_value = instruction.c_as_boolean();
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
                    let compare_to = instruction.a_as_boolean();

                    if boolean == compare_to {
                        self.ip += 1;
                    } else {
                        let jump_distance = jump.a();
                        let is_positive = jump.a_as_boolean();
                        let new_ip = if is_positive {
                            self.ip + jump_distance as usize
                        } else {
                            self.ip - jump_distance as usize
                        };

                        self.ip = new_ip;
                    }
                }
                Operation::Less => {
                    let jump = self.chunk.get_instruction(self.ip, position)?.0;

                    debug_assert_eq!(jump.operation(), Operation::Jump);

                    let (left, right) = get_arguments(self, instruction, position)?;

                    let boolean = left
                        .less_than(right)
                        .map_err(|error| VmError::Value { error, position })?
                        .as_boolean()
                        .ok_or_else(|| VmError::ExpectedBoolean {
                            found: left.clone(),
                            position,
                        })?;
                    let compare_to = instruction.a_as_boolean();

                    if boolean == compare_to {
                        self.ip += 1;
                    } else {
                        let jump_distance = jump.a();
                        let is_positive = jump.a_as_boolean();
                        let new_ip = if is_positive {
                            self.ip + jump_distance as usize
                        } else {
                            self.ip - jump_distance as usize
                        };

                        self.ip = new_ip;
                    }
                }
                Operation::LessEqual => {
                    let jump = self.chunk.get_instruction(self.ip, position)?.0;

                    debug_assert_eq!(jump.operation(), Operation::Jump);

                    let (left, right) = get_arguments(self, instruction, position)?;
                    let boolean = left
                        .less_than_or_equal(right)
                        .map_err(|error| VmError::Value { error, position })?
                        .as_boolean()
                        .ok_or_else(|| VmError::ExpectedBoolean {
                            found: left.clone(),
                            position,
                        })?;
                    let compare_to = instruction.a_as_boolean();

                    if boolean == compare_to {
                        self.ip += 1;
                    } else {
                        let jump_distance = jump.a();
                        let is_positive = jump.a_as_boolean();
                        let new_ip = if is_positive {
                            self.ip + jump_distance as usize
                        } else {
                            self.ip - jump_distance as usize
                        };

                        self.ip = new_ip;
                    }
                }
                Operation::Negate => {
                    let value = if instruction.b_is_constant() {
                        self.chunk.get_constant(instruction.b(), position)?
                    } else {
                        self.get(instruction.b(), position)?
                    };
                    let negated = value
                        .negate()
                        .map_err(|error| VmError::Value { error, position })?;

                    self.insert(negated, instruction.a(), position)?;
                }
                Operation::Not => {
                    let value = if instruction.b_is_constant() {
                        self.chunk.get_constant(instruction.b(), position)?
                    } else {
                        self.get(instruction.b(), position)?
                    };
                    let not = value
                        .not()
                        .map_err(|error| VmError::Value { error, position })?;

                    self.insert(not, instruction.a(), position)?;
                }
                Operation::Jump => {
                    let offset = instruction.b();
                    let is_positive = instruction.c_as_boolean();
                    let new_ip = if is_positive {
                        self.ip + offset as usize
                    } else {
                        self.ip - offset as usize
                    };

                    self.ip = new_ip;
                }
                Operation::Return => {
                    return if let Some(Some(value)) = self.register_stack.pop() {
                        Ok(Some(value))
                    } else {
                        Ok(None)
                    };
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

    fn _clone_mutable(&mut self, index: u8, position: Span) -> Result<Value, VmError> {
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

    fn _clone_as_variable(&mut self, local: Local, position: Span) -> Result<Value, VmError> {
        let index = if let Some(index) = local.register_index {
            index
        } else {
            return Err(VmError::UndefinedVariable {
                identifier: local.identifier,
                position,
            });
        };
        let clone_result = if local.mutable {
            self._clone_mutable(index, position)
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
