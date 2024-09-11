use crate::{
    dust_error::AnnotatedError, parse, Chunk, ChunkError, DustError, Identifier, Instruction, Span,
    Value, ValueError,
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
    stack: Vec<Value>,
}

impl Vm {
    const STACK_SIZE: usize = 256;

    pub fn new(chunk: Chunk) -> Self {
        Self {
            chunk,
            ip: 0,
            stack: Vec::with_capacity(Self::STACK_SIZE),
        }
    }

    pub fn run(&mut self) -> Result<Option<Value>, VmError> {
        let mut current_position = Span(0, 0);

        while let Ok((byte, position)) = self.read(current_position).copied() {
            current_position = position;

            let instruction = Instruction::from_byte(byte)
                .ok_or_else(|| VmError::InvalidInstruction(byte, position))?;

            log::trace!("Running instruction {instruction} at {position}");

            match instruction {
                Instruction::Constant => {
                    let (argument, _) = *self.read(position)?;
                    let value = self.chunk.use_constant(argument, position)?;

                    log::trace!("Pushing constant {value}");

                    self.push(value, position)?;
                }
                Instruction::Return => {
                    let value = self.pop(position)?;

                    log::trace!("Returning {value}");

                    return Ok(Some(value));
                }
                Instruction::Pop => {
                    let value = self.pop(position)?;

                    log::trace!("Popping {value:?}");
                }

                // Variables
                Instruction::DeclareVariable => {
                    let (argument, _) = *self.read(position)?;
                    let identifier = self
                        .chunk
                        .get_identifier(argument)
                        .ok_or_else(|| VmError::UndeclaredVariable { position })?;
                    let value = self.stack.remove(argument as usize);

                    log::trace!("Declaring {identifier} as value {value}",);

                    self.push(value, position)?;
                }
                Instruction::GetVariable => {
                    let (argument, _) = *self.read(position)?;
                    let identifier = self
                        .chunk
                        .get_identifier(argument)
                        .ok_or_else(|| VmError::UndeclaredVariable { position })?;
                    let value = self.stack.remove(argument as usize);

                    log::trace!("Getting {identifier} as value {value}",);

                    self.push(value, position)?;
                }
                Instruction::SetVariable => {
                    let (argument, _) = *self.read(position)?;
                    let identifier = self
                        .chunk
                        .get_identifier(argument)
                        .ok_or_else(|| VmError::UndeclaredVariable { position })?
                        .clone();

                    if !self.chunk.contains_identifier(&identifier) {
                        return Err(VmError::UndefinedVariable {
                            identifier: identifier.clone(),
                            position,
                        });
                    }

                    let value = self.pop(position)?;

                    log::trace!("Setting {identifier} to {value}");

                    self.stack[argument as usize] = value;
                }

                // Unary
                Instruction::Negate => {
                    let negated = self
                        .pop(position)?
                        .negate()
                        .map_err(|error| VmError::Value { error, position })?;

                    self.push(negated, position)?;
                }
                Instruction::Not => {
                    let not = self
                        .pop(position)?
                        .not()
                        .map_err(|error| VmError::Value { error, position })?;

                    self.push(not, position)?;
                }

                // Binary
                Instruction::Add => {
                    let right = self.pop(position)?;
                    let left = self.pop(position)?;
                    let sum = left
                        .add(&right)
                        .map_err(|error| VmError::Value { error, position })?;

                    self.push(sum, position)?;
                }
                Instruction::Subtract => {
                    let right = self.pop(position)?;
                    let left = self.pop(position)?;
                    let difference = left
                        .subtract(&right)
                        .map_err(|error| VmError::Value { error, position })?;

                    self.push(difference, position)?;
                }
                Instruction::Multiply => {
                    let right = self.pop(position)?;
                    let left = self.pop(position)?;
                    let product = left
                        .multiply(&right)
                        .map_err(|error| VmError::Value { error, position })?;

                    self.push(product, position)?;
                }
                Instruction::Divide => {
                    let right = self.pop(position)?;
                    let left = self.pop(position)?;
                    let quotient = left
                        .divide(&right)
                        .map_err(|error| VmError::Value { error, position })?;

                    self.push(quotient, position)?;
                }
                Instruction::Greater => {
                    let right = self.pop(position)?;
                    let left = self.pop(position)?;
                    let greater = left
                        .greater_than(&right)
                        .map_err(|error| VmError::Value { error, position })?;

                    self.push(greater, position)?;
                }
                Instruction::Less => {
                    let right = self.pop(position)?;
                    let left = self.pop(position)?;
                    let less = left
                        .less_than(&right)
                        .map_err(|error| VmError::Value { error, position })?;

                    self.push(less, position)?;
                }
                Instruction::GreaterEqual => {
                    let right = self.pop(position)?;
                    let left = self.pop(position)?;
                    let greater_equal = left
                        .greater_than_or_equal(&right)
                        .map_err(|error| VmError::Value { error, position })?;

                    self.push(greater_equal, position)?;
                }
                Instruction::LessEqual => {
                    let right = self.pop(position)?;
                    let left = self.pop(position)?;
                    let less_equal = left
                        .less_than_or_equal(&right)
                        .map_err(|error| VmError::Value { error, position })?;

                    self.push(less_equal, position)?;
                }
                Instruction::Equal => {
                    let right = self.pop(position)?;
                    let left = self.pop(position)?;
                    let equal = left
                        .equal(&right)
                        .map_err(|error| VmError::Value { error, position })?;

                    self.push(equal, position)?;
                }
                Instruction::NotEqual => {
                    let right = self.pop(position)?;
                    let left = self.pop(position)?;
                    let not_equal = left
                        .not_equal(&right)
                        .map_err(|error| VmError::Value { error, position })?;

                    self.push(not_equal, position)?;
                }
                Instruction::And => {
                    let right = self.pop(position)?;
                    let left = self.pop(position)?;
                    let and = left
                        .and(&right)
                        .map_err(|error| VmError::Value { error, position })?;

                    self.push(and, position)?;
                }
                Instruction::Or => {
                    let right = self.pop(position)?;
                    let left = self.pop(position)?;
                    let or = left
                        .or(&right)
                        .map_err(|error| VmError::Value { error, position })?;

                    self.push(or, position)?;
                }
            }
        }

        Ok(None)
    }

    fn push(&mut self, value: Value, position: Span) -> Result<(), VmError> {
        if self.stack.len() == Self::STACK_SIZE {
            Err(VmError::StackOverflow(position))
        } else {
            let value = if value.is_raw() {
                value.into_reference()
            } else {
                value
            };

            self.stack.push(value);

            Ok(())
        }
    }

    fn pop(&mut self, position: Span) -> Result<Value, VmError> {
        if let Some(stacked) = self.stack.pop() {
            Ok(stacked)
        } else {
            Err(VmError::StackUnderflow(position))
        }
    }

    fn read(&mut self, position: Span) -> Result<&(u8, Span), VmError> {
        let current = self.chunk.get_code(self.ip, position)?;

        self.ip += 1;

        Ok(current)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum VmError {
    InvalidInstruction(u8, Span),
    StackOverflow(Span),
    StackUnderflow(Span),
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
            Self::InvalidInstruction(_, _) => "Invalid instruction",
            Self::StackOverflow(_) => "Stack overflow",
            Self::StackUnderflow(_) => "Stack underflow",
            Self::UndeclaredVariable { .. } => "Undeclared variable",
            Self::UndefinedVariable { .. } => "Undefined variable",
            Self::Chunk(_) => "Chunk error",
            Self::Value { .. } => "Value error",
        }
    }

    fn details(&self) -> Option<String> {
        match self {
            Self::InvalidInstruction(byte, _) => Some(format!(
                "The byte {byte} does not correspond to a valid instruction"
            )),
            Self::StackOverflow(position) => Some(format!("Stack overflow at {position}")),
            Self::StackUnderflow(position) => Some(format!("Stack underflow at {position}")),
            Self::UndeclaredVariable { .. } => Some("Variable is not declared".to_string()),
            Self::UndefinedVariable { identifier, .. } => {
                Some(format!("{identifier} is not in scope"))
            }
            Self::Chunk(error) => error.details(),
            Self::Value { error, .. } => Some(error.to_string()),
        }
    }

    fn position(&self) -> Span {
        match self {
            Self::InvalidInstruction(_, position) => *position,
            Self::StackUnderflow(position) => *position,
            Self::StackOverflow(position) => *position,
            Self::UndeclaredVariable { position } => *position,
            Self::UndefinedVariable { position, .. } => *position,
            Self::Chunk(error) => error.position(),
            Self::Value { position, .. } => *position,
        }
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;

    #[test]
    fn negation() {
        let mut chunk = Chunk::new();
        let dummy_position = Span(0, 0);
        let constant = chunk
            .push_constant(Value::integer(42), dummy_position)
            .unwrap();

        chunk.push_code(Instruction::Constant as u8, dummy_position);
        chunk.push_code(constant, dummy_position);
        chunk.push_code(Instruction::Negate as u8, dummy_position);
        chunk.push_code(Instruction::Return as u8, dummy_position);

        let mut vm = Vm::new(chunk);
        let result = vm.run();

        assert_eq!(result, Ok(Some(Value::integer(-42))));
    }

    #[test]
    fn addition() {
        let mut chunk = Chunk::new();
        let dummy_position = Span(0, 0);
        let left = chunk
            .push_constant(Value::integer(42), dummy_position)
            .unwrap();
        let right = chunk
            .push_constant(Value::integer(23), dummy_position)
            .unwrap();

        chunk.push_code(Instruction::Constant, dummy_position);
        chunk.push_code(left, dummy_position);
        chunk.push_code(Instruction::Constant, dummy_position);
        chunk.push_code(right, dummy_position);
        chunk.push_code(Instruction::Add, dummy_position);
        chunk.push_code(Instruction::Return, dummy_position);

        let mut vm = Vm::new(chunk);
        let result = vm.run();

        assert_eq!(result, Ok(Some(Value::integer(65))));
    }

    #[test]
    fn subtraction() {
        let mut chunk = Chunk::new();
        let dummy_position = Span(0, 0);
        let left = chunk
            .push_constant(Value::integer(42), dummy_position)
            .unwrap();
        let right = chunk
            .push_constant(Value::integer(23), dummy_position)
            .unwrap();

        chunk.push_code(Instruction::Constant, dummy_position);
        chunk.push_code(left, dummy_position);
        chunk.push_code(Instruction::Constant, dummy_position);
        chunk.push_code(right, dummy_position);
        chunk.push_code(Instruction::Subtract, dummy_position);
        chunk.push_code(Instruction::Return, dummy_position);

        let mut vm = Vm::new(chunk);
        let result = vm.run();

        assert_eq!(result, Ok(Some(Value::integer(19))));
    }

    #[test]
    fn multiplication() {
        let mut chunk = Chunk::new();
        let dummy_position = Span(0, 0);
        let left = chunk
            .push_constant(Value::integer(42), dummy_position)
            .unwrap();
        let right = chunk
            .push_constant(Value::integer(23), dummy_position)
            .unwrap();

        chunk.push_code(Instruction::Constant, dummy_position);
        chunk.push_code(left, dummy_position);
        chunk.push_code(Instruction::Constant, dummy_position);
        chunk.push_code(right, dummy_position);
        chunk.push_code(Instruction::Multiply, dummy_position);
        chunk.push_code(Instruction::Return, dummy_position);

        let mut vm = Vm::new(chunk);
        let result = vm.run();

        assert_eq!(result, Ok(Some(Value::integer(966))));
    }

    #[test]

    fn division() {
        let mut chunk = Chunk::new();
        let dummy_position = Span(0, 0);
        let left = chunk
            .push_constant(Value::integer(42), dummy_position)
            .unwrap();
        let right = chunk
            .push_constant(Value::integer(23), dummy_position)
            .unwrap();

        chunk.push_code(Instruction::Constant, dummy_position);
        chunk.push_code(left, dummy_position);
        chunk.push_code(Instruction::Constant, dummy_position);
        chunk.push_code(right, dummy_position);
        chunk.push_code(Instruction::Divide, dummy_position);
        chunk.push_code(Instruction::Return, dummy_position);

        let mut vm = Vm::new(chunk);
        let result = vm.run();

        assert_eq!(result, Ok(Some(Value::integer(1))));
    }
}
