use crate::{
    dust_error::AnnotatedError, parse, Chunk, ChunkError, DustError, Identifier, Instruction,
    Operation, Span, Value, ValueError,
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
        while let Ok((instruction, position)) = self.read(Span(0, 0)).copied() {
            log::trace!("Running instruction {instruction} at {position}");

            match instruction.operation() {
                Operation::Move => {
                    let from = instruction.first_argument();
                    let to = instruction.destination();
                    let value = self.clone(from, position)?;

                    self.insert(value, to, position)?;
                }
                Operation::Close => todo!(),
                Operation::LoadConstant => {
                    let to_register = instruction.destination();
                    let from_constant = instruction.first_argument();
                    let value = self.chunk.take_constant(from_constant, position)?;

                    self.insert(value, to_register, position)?;
                }
                Operation::LoadList => {
                    let to_register = instruction.destination();
                    let length = instruction.first_argument();
                    let first_register = to_register - length;
                    let last_register = to_register - 1;

                    let mut list = Vec::with_capacity(length as usize);

                    for register_index in first_register..=last_register {
                        let value = self.clone(register_index, position)?;

                        list.push(value);
                    }

                    self.insert(Value::list(list), to_register, position)?;
                }
                Operation::DeclareLocal => {
                    let from_register = instruction.destination();
                    let to_local = instruction.first_argument();

                    self.chunk.define_local(to_local, from_register, position)?;
                }
                Operation::GetLocal => {
                    let register_index = instruction.destination();
                    let local_index = instruction.first_argument();
                    let local = self.chunk.get_local(local_index, position)?;
                    let value = if let Some(value_index) = &local.register_index {
                        self.clone(*value_index, position)?
                    } else {
                        return Err(VmError::UndefinedVariable {
                            identifier: local.identifier.clone(),
                            position,
                        });
                    };

                    self.insert(value, register_index, position)?;
                }
                Operation::SetLocal => {
                    let from_register = instruction.destination();
                    let to_local = instruction.first_argument();

                    self.chunk.define_local(to_local, from_register, position)?;
                }
                Operation::Add => {
                    let left = self.take_constant_or_clone_register(
                        instruction.first_argument(),
                        instruction.first_argument_is_constant(),
                        position,
                    )?;
                    let right = self.take_constant_or_clone_register(
                        instruction.second_argument(),
                        instruction.second_argument_is_constant(),
                        position,
                    )?;
                    let sum = left
                        .add(&right)
                        .map_err(|error| VmError::Value { error, position })?;

                    self.insert(sum, instruction.destination(), position)?;
                }
                Operation::Subtract => {
                    let left = self.take_constant_or_clone_register(
                        instruction.first_argument(),
                        instruction.first_argument_is_constant(),
                        position,
                    )?;
                    let right = self.take_constant_or_clone_register(
                        instruction.second_argument(),
                        instruction.second_argument_is_constant(),
                        position,
                    )?;
                    let difference = left
                        .subtract(&right)
                        .map_err(|error| VmError::Value { error, position })?;

                    self.insert(difference, instruction.destination(), position)?;
                }
                Operation::Multiply => {
                    let left = self.take_constant_or_clone_register(
                        instruction.first_argument(),
                        instruction.first_argument_is_constant(),
                        position,
                    )?;
                    let right = self.take_constant_or_clone_register(
                        instruction.second_argument(),
                        instruction.second_argument_is_constant(),
                        position,
                    )?;
                    let product = left
                        .multiply(&right)
                        .map_err(|error| VmError::Value { error, position })?;

                    self.insert(product, instruction.destination(), position)?;
                }
                Operation::Divide => {
                    let left = self.take_constant_or_clone_register(
                        instruction.first_argument(),
                        instruction.first_argument_is_constant(),
                        position,
                    )?;
                    let right = self.take_constant_or_clone_register(
                        instruction.second_argument(),
                        instruction.second_argument_is_constant(),
                        position,
                    )?;
                    let quotient = left
                        .divide(&right)
                        .map_err(|error| VmError::Value { error, position })?;

                    self.insert(quotient, instruction.destination(), position)?;
                }
                Operation::Modulo => {
                    let left = self.take_constant_or_clone_register(
                        instruction.first_argument(),
                        instruction.first_argument_is_constant(),
                        position,
                    )?;
                    let right = self.take_constant_or_clone_register(
                        instruction.second_argument(),
                        instruction.second_argument_is_constant(),
                        position,
                    )?;
                    let remainder = left
                        .modulo(&right)
                        .map_err(|error| VmError::Value { error, position })?;

                    self.insert(remainder, instruction.destination(), position)?;
                }
                Operation::And => {
                    let left = self.take_constant_or_clone_register(
                        instruction.first_argument(),
                        instruction.first_argument_is_constant(),
                        position,
                    )?;
                    let right = self.take_constant_or_clone_register(
                        instruction.second_argument(),
                        instruction.second_argument_is_constant(),
                        position,
                    )?;
                    let result = left
                        .and(&right)
                        .map_err(|error| VmError::Value { error, position })?;

                    self.insert(result, instruction.destination(), position)?;
                }
                Operation::Or => {
                    let left = self.take_constant_or_clone_register(
                        instruction.first_argument(),
                        instruction.first_argument_is_constant(),
                        position,
                    )?;
                    let right = self.take_constant_or_clone_register(
                        instruction.second_argument(),
                        instruction.second_argument_is_constant(),
                        position,
                    )?;
                    let result = left
                        .or(&right)
                        .map_err(|error| VmError::Value { error, position })?;

                    self.insert(result, instruction.destination(), position)?;
                }
                Operation::Negate => {
                    let value = self.take_constant_or_clone_register(
                        instruction.first_argument(),
                        instruction.first_argument_is_constant(),
                        position,
                    )?;
                    let negated = value
                        .negate()
                        .map_err(|error| VmError::Value { error, position })?;

                    self.insert(negated, instruction.destination(), position)?;
                }
                Operation::Not => {
                    let value = self.take_constant_or_clone_register(
                        instruction.first_argument(),
                        instruction.first_argument_is_constant(),
                        position,
                    )?;
                    let result = value
                        .not()
                        .map_err(|error| VmError::Value { error, position })?;

                    self.insert(result, instruction.destination(), position)?;
                }
                Operation::Return => {
                    let value = self.pop(position)?;

                    return Ok(Some(value));
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

    fn take_constant_or_clone_register(
        &mut self,
        index: u8,
        is_constant: bool,
        position: Span,
    ) -> Result<Value, VmError> {
        if is_constant {
            Ok(self.chunk.take_constant(index, position)?)
        } else {
            self.clone(index, position)
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
    UndeclaredVariable {
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
    fn from(v: ChunkError) -> Self {
        Self::Chunk(v)
    }
}

impl AnnotatedError for VmError {
    fn title() -> &'static str {
        "Runtime Error"
    }

    fn description(&self) -> &'static str {
        match self {
            Self::EmptyRegister { .. } => "Empty register",
            Self::RegisterIndexOutOfBounds { .. } => "Register index out of bounds",
            Self::InvalidInstruction { .. } => "Invalid instruction",
            Self::StackOverflow { .. } => "Stack overflow",
            Self::StackUnderflow { .. } => "Stack underflow",
            Self::UndeclaredVariable { .. } => "Undeclared variable",
            Self::UndefinedVariable { .. } => "Undefined variable",
            Self::Chunk(error) => error.description(),
            Self::Value { .. } => "Value error",
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
            Self::RegisterIndexOutOfBounds { position } => *position,
            Self::InvalidInstruction { position, .. } => *position,
            Self::StackUnderflow { position } => *position,
            Self::StackOverflow { position } => *position,
            Self::UndeclaredVariable { position } => *position,
            Self::UndefinedVariable { position, .. } => *position,
            Self::Chunk(error) => error.position(),
            Self::Value { position, .. } => *position,
        }
    }
}
