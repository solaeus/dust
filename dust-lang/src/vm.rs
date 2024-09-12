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

            match instruction.operation {
                Operation::Move => todo!(),
                Operation::Close => todo!(),
                Operation::LoadConstant => {
                    let constant_index = u16::from_le_bytes(instruction.arguments) as usize;
                    let value = self.chunk.use_constant(constant_index, position)?;

                    self.insert(value, instruction.destination as usize, position)?;
                }
                Operation::DeclareLocal => {
                    let register_index = instruction.destination as usize;
                    let local_index = u16::from_le_bytes(instruction.arguments) as usize;
                    let value = self.clone(register_index, position)?;

                    self.chunk.define_local(local_index, value, position)?;
                }
                Operation::GetLocal => {
                    let register_index = instruction.destination as usize;
                    let local_index = u16::from_le_bytes(instruction.arguments) as usize;
                    let local = self.chunk.get_local(local_index, position)?;
                    let value = if let Some(value) = &local.value {
                        value.clone()
                    } else {
                        return Err(VmError::UndefinedVariable {
                            identifier: local.identifier.clone(),
                            position,
                        });
                    };

                    self.insert(value, register_index, position)?;
                }
                Operation::SetLocal => todo!(),
                Operation::Add => {
                    let left =
                        self.take_or_use_constant(instruction.arguments[0] as usize, position)?;
                    let right =
                        self.take_or_use_constant(instruction.arguments[1] as usize, position)?;
                    let sum = left
                        .add(&right)
                        .map_err(|error| VmError::Value { error, position })?;

                    self.insert(sum, instruction.destination as usize, position)?;
                }
                Operation::Subtract => {
                    let left =
                        self.take_or_use_constant(instruction.arguments[0] as usize, position)?;
                    let right =
                        self.take_or_use_constant(instruction.arguments[1] as usize, position)?;
                    let difference = left
                        .subtract(&right)
                        .map_err(|error| VmError::Value { error, position })?;

                    self.insert(difference, instruction.destination as usize, position)?;
                }
                Operation::Multiply => {
                    let left =
                        self.take_or_use_constant(instruction.arguments[0] as usize, position)?;
                    let right =
                        self.take_or_use_constant(instruction.arguments[1] as usize, position)?;
                    let product = left
                        .multiply(&right)
                        .map_err(|error| VmError::Value { error, position })?;

                    self.insert(product, instruction.destination as usize, position)?;
                }
                Operation::Divide => {
                    let left =
                        self.take_or_use_constant(instruction.arguments[0] as usize, position)?;
                    let right =
                        self.take_or_use_constant(instruction.arguments[1] as usize, position)?;
                    let quotient = left
                        .divide(&right)
                        .map_err(|error| VmError::Value { error, position })?;

                    self.insert(quotient, instruction.destination as usize, position)?;
                }
                Operation::Negate => {
                    let value =
                        self.take_or_use_constant(instruction.arguments[0] as usize, position)?;
                    let negated = value
                        .negate()
                        .map_err(|error| VmError::Value { error, position })?;

                    self.insert(negated, instruction.destination as usize, position)?;
                }
                Operation::Return => {
                    let value = self.pop(position)?;

                    return Ok(Some(value));
                }
            }
        }

        Ok(None)
    }

    fn insert(&mut self, value: Value, index: usize, position: Span) -> Result<(), VmError> {
        if self.register_stack.len() == Self::STACK_LIMIT {
            Err(VmError::StackOverflow { position })
        } else {
            while index >= self.register_stack.len() {
                self.register_stack.push(None);
            }

            self.register_stack[index] = Some(value);

            Ok(())
        }
    }

    fn clone(&mut self, index: usize, position: Span) -> Result<Value, VmError> {
        if let Some(register) = self.register_stack.get_mut(index) {
            if let Some(mut value) = register.take() {
                if value.is_raw() {
                    value = value.into_reference();
                }

                Ok(value.clone())
            } else {
                Err(VmError::EmptyRegister { index, position })
            }
        } else {
            Err(VmError::RegisterIndexOutOfBounds { position })
        }
    }

    fn take_or_use_constant(&mut self, index: usize, position: Span) -> Result<Value, VmError> {
        if let Ok(value) = self.clone(index, position) {
            Ok(value)
        } else {
            let value = self.chunk.use_constant(index, position)?;

            Ok(value)
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
            Self::Chunk(_) => "Chunk error",
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
