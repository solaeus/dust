use std::rc::Rc;

use crate::{
    dust_error::AnnotatedError, parse, Chunk, ChunkError, DustError, Identifier, Instruction, Span,
    Value, ValueError, ValueLocation,
};

pub fn run(source: &str) -> Result<Option<Value>, DustError> {
    let chunk = parse(source)?;

    let mut vm = Vm::new(chunk);

    vm.run()
        .map_err(|error| DustError::Runtime { error, source })
}

#[derive(Debug, Eq, PartialEq)]
pub struct Vm {
    chunk: Rc<Chunk>,
    ip: usize,
    stack: Vec<StackedValue>,
}

impl Vm {
    const STACK_SIZE: usize = 256;

    pub fn new(chunk: Chunk) -> Self {
        Self {
            chunk: Rc::new(chunk),
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
                    let value = self.chunk.get_constant(argument, position)?.clone();

                    log::trace!("Pushing constant {value}");

                    self.push_runtime_value(value, position)?;
                }
                Instruction::Return => {
                    let stacked = self.pop(position)?;
                    let value = match stacked {
                        StackedValue::Runtime(value) => value,
                        StackedValue::Constant(index) => Rc::get_mut(&mut self.chunk)
                            .unwrap()
                            .remove_constant(index, position)?,
                    };

                    log::trace!("Returning {value}");

                    return Ok(Some(value));
                }
                Instruction::Pop => {
                    let value = self.pop(position)?;

                    log::trace!("Popping {value:?}");
                }

                // Variables
                Instruction::DefineVariable => {
                    let (argument, _) = *self.read(position)?;
                    let identifier = self.chunk.get_identifier(argument)?.clone();
                    let stack_index_option = self.chunk.resolve_local(&identifier);

                    if let Some(index) = stack_index_option {
                        let value = self.stack[index as usize]
                            .to_value(&self.chunk, position)?
                            .clone();

                        log::trace!("Defining {identifier} as value {value}");

                        self.push_runtime_value(value, position)?;
                    } else {
                        return Err(VmError::UndefinedVariable {
                            identifier,
                            position,
                        });
                    }
                }
                Instruction::GetVariable => {
                    let (argument, _) = *self.read(position)?;
                    let value = self.pop(position)?.to_value(&self.chunk, position)?.clone();

                    log::trace!(
                        "Getting {} as value {value}",
                        self.chunk.get_identifier(argument)?,
                    );

                    self.push_runtime_value(value, position)?;
                }
                Instruction::SetVariable => {
                    let (argument, _) = *self.read(position)?;
                    let identifier = self.chunk.get_identifier(argument)?.clone();

                    if !self.chunk.contains_identifier(&identifier) {
                        return Err(VmError::UndefinedVariable {
                            identifier,
                            position,
                        });
                    }

                    let value = self.stack[argument as usize]
                        .to_value(&self.chunk, position)?
                        .clone();

                    log::trace!("Setting {identifier} to {value}");

                    self.push_runtime_value(value, position)?;
                }

                // Unary
                Instruction::Negate => {
                    let negated = self
                        .pop(position)?
                        .to_value(&self.chunk, position)?
                        .negate()
                        .map_err(|error| VmError::Value { error, position })?;

                    self.push_runtime_value(negated, position)?;
                }
                Instruction::Not => {
                    let not = self
                        .pop(position)?
                        .to_value(&self.chunk, position)?
                        .not()
                        .map_err(|error| VmError::Value { error, position })?;

                    self.push_runtime_value(not, position)?;
                }

                // Binary
                Instruction::Add => {
                    let chunk = self.chunk.clone();
                    let right_stacked = self.pop(position)?;
                    let right = right_stacked.to_value(chunk.as_ref(), position)?;
                    let left_stacked = self.pop(position)?;
                    let left = left_stacked.to_value(&self.chunk, position)?;
                    let sum = left
                        .add(right)
                        .map_err(|error| VmError::Value { error, position })?;

                    self.push_runtime_value(sum, position)?;
                }
                Instruction::Subtract => {
                    let chunk = self.chunk.clone();
                    let right_stacked = self.pop(position)?;
                    let right = right_stacked.to_value(chunk.as_ref(), position)?;
                    let left_stacked = self.pop(position)?;
                    let left = left_stacked.to_value(&self.chunk, position)?;
                    let difference = left
                        .subtract(right)
                        .map_err(|error| VmError::Value { error, position })?;

                    self.push_runtime_value(difference, position)?;
                }
                Instruction::Multiply => {
                    let chunk = self.chunk.clone();
                    let right_stacked = self.pop(position)?;
                    let right = right_stacked.to_value(chunk.as_ref(), position)?;
                    let left_stacked = self.pop(position)?;
                    let left = left_stacked.to_value(&self.chunk, position)?;
                    let product = left
                        .multiply(right)
                        .map_err(|error| VmError::Value { error, position })?;

                    self.push_runtime_value(product, position)?;
                }
                Instruction::Divide => {
                    let chunk = self.chunk.clone();
                    let right_stacked = self.pop(position)?;
                    let right = right_stacked.to_value(chunk.as_ref(), position)?;
                    let left_stacked = self.pop(position)?;
                    let left = left_stacked.to_value(&self.chunk, position)?;
                    let quotient = left
                        .divide(right)
                        .map_err(|error| VmError::Value { error, position })?;

                    self.push_runtime_value(quotient, position)?;
                }
                Instruction::Greater => {
                    let chunk = self.chunk.clone();
                    let right_stacked = self.pop(position)?;
                    let right = right_stacked.to_value(chunk.as_ref(), position)?;
                    let left_stacked = self.pop(position)?;
                    let left = left_stacked.to_value(&self.chunk, position)?;
                    let greater = left
                        .greater_than(right)
                        .map_err(|error| VmError::Value { error, position })?;

                    self.push_runtime_value(greater, position)?;
                }
                Instruction::Less => {
                    let chunk = self.chunk.clone();
                    let right_stacked = self.pop(position)?;
                    let right = right_stacked.to_value(chunk.as_ref(), position)?;
                    let left_stacked = self.pop(position)?;
                    let left = left_stacked.to_value(&self.chunk, position)?;
                    let less = left
                        .less_than(right)
                        .map_err(|error| VmError::Value { error, position })?;

                    self.push_runtime_value(less, position)?;
                }
                Instruction::GreaterEqual => {
                    let chunk = self.chunk.clone();
                    let right_stacked = self.pop(position)?;
                    let right = right_stacked.to_value(chunk.as_ref(), position)?;
                    let left_stacked = self.pop(position)?;
                    let left = left_stacked.to_value(&self.chunk, position)?;
                    let greater_equal = left
                        .greater_than_or_equal(right)
                        .map_err(|error| VmError::Value { error, position })?;

                    self.push_runtime_value(greater_equal, position)?;
                }
                Instruction::LessEqual => {
                    let chunk = self.chunk.clone();
                    let right_stacked = self.pop(position)?;
                    let right = right_stacked.to_value(chunk.as_ref(), position)?;
                    let left_stacked = self.pop(position)?;
                    let left = left_stacked.to_value(&self.chunk, position)?;
                    let less_equal = left
                        .less_than_or_equal(right)
                        .map_err(|error| VmError::Value { error, position })?;

                    self.push_runtime_value(less_equal, position)?;
                }
                Instruction::Equal => {
                    let chunk = self.chunk.clone();
                    let right_stacked = self.pop(position)?;
                    let right = right_stacked.to_value(chunk.as_ref(), position)?;
                    let left_stacked = self.pop(position)?;
                    let left = left_stacked.to_value(&self.chunk, position)?;
                    let equal = left
                        .equal(right)
                        .map_err(|error| VmError::Value { error, position })?;

                    self.push_runtime_value(equal, position)?;
                }
                Instruction::NotEqual => {
                    let chunk = self.chunk.clone();
                    let right_stacked = self.pop(position)?;
                    let right = right_stacked.to_value(chunk.as_ref(), position)?;
                    let left_stacked = self.pop(position)?;
                    let left = left_stacked.to_value(&self.chunk, position)?;
                    let not_equal = left
                        .not_equal(right)
                        .map_err(|error| VmError::Value { error, position })?;

                    self.push_runtime_value(not_equal, position)?;
                }
                Instruction::And => {
                    let chunk = self.chunk.clone();
                    let right_stacked = self.pop(position)?;
                    let right = right_stacked.to_value(chunk.as_ref(), position)?;
                    let left_stacked = self.pop(position)?;
                    let left = left_stacked.to_value(&self.chunk, position)?;
                    let and = left
                        .and(right)
                        .map_err(|error| VmError::Value { error, position })?;

                    self.push_runtime_value(and, position)?;
                }
                Instruction::Or => {
                    let chunk = self.chunk.clone();
                    let right_stacked = self.pop(position)?;
                    let right = right_stacked.to_value(chunk.as_ref(), position)?;
                    let left_stacked = self.pop(position)?;
                    let left = left_stacked.to_value(&self.chunk, position)?;
                    let or = left
                        .or(right)
                        .map_err(|error| VmError::Value { error, position })?;

                    self.push_runtime_value(or, position)?;
                }
            }
        }

        Ok(None)
    }

    fn push_runtime_value(&mut self, value: Value, position: Span) -> Result<(), VmError> {
        if self.stack.len() == Self::STACK_SIZE {
            Err(VmError::StackOverflow(position))
        } else {
            let value = if value.is_raw() {
                value.into_reference()
            } else {
                value
            };

            self.stack.push(StackedValue::Runtime(value));

            Ok(())
        }
    }

    fn push_constant_value(&mut self, index: u8, position: Span) -> Result<(), VmError> {
        if self.stack.len() == Self::STACK_SIZE {
            Err(VmError::StackOverflow(position))
        } else {
            self.stack.push(StackedValue::Constant(index));

            Ok(())
        }
    }

    fn pop(&mut self, position: Span) -> Result<StackedValue, VmError> {
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

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum StackedValue {
    Runtime(Value),
    Constant(u8),
}

impl StackedValue {
    fn to_value<'a>(&'a self, chunk: &'a Chunk, position: Span) -> Result<&'a Value, VmError> {
        match self {
            Self::Runtime(value) => Ok(value),
            Self::Constant(index) => Ok(chunk.get_constant(*index, position)?),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum VmError {
    InvalidInstruction(u8, Span),
    StackOverflow(Span),
    StackUnderflow(Span),
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
            Self::UndefinedVariable { identifier, .. } => {
                Some(format!("{identifier} is not in scope"))
            }

            Self::Chunk(error) => Some(error.to_string()),
            Self::Value { error, .. } => Some(error.to_string()),
        }
    }

    fn position(&self) -> Span {
        match self {
            Self::InvalidInstruction(_, position) => *position,
            Self::StackUnderflow(position) => *position,
            Self::StackOverflow(position) => *position,
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
        let constant = chunk.push_constant(Value::integer(42)).unwrap();

        chunk.push_code(Instruction::Constant as u8, Span(0, 1));
        chunk.push_code(constant, Span(2, 3));
        chunk.push_code(Instruction::Negate as u8, Span(4, 5));
        chunk.push_code(Instruction::Return as u8, Span(2, 3));

        let mut vm = Vm::new(chunk);
        let result = vm.run();

        assert_eq!(result, Ok(Some(Value::integer(-42))));
    }

    #[test]
    fn addition() {
        let mut chunk = Chunk::new();
        let left = chunk.push_constant(Value::integer(42)).unwrap();
        let right = chunk.push_constant(Value::integer(23)).unwrap();

        chunk.push_code(Instruction::Constant as u8, Span(0, 1));
        chunk.push_code(left, Span(2, 3));
        chunk.push_code(Instruction::Constant as u8, Span(4, 5));
        chunk.push_code(right, Span(6, 7));
        chunk.push_code(Instruction::Add as u8, Span(8, 9));
        chunk.push_code(Instruction::Return as u8, Span(10, 11));

        let mut vm = Vm::new(chunk);
        let result = vm.run();

        assert_eq!(result, Ok(Some(Value::integer(65))));
    }

    #[test]
    fn subtraction() {
        let mut chunk = Chunk::new();
        let left = chunk.push_constant(Value::integer(42)).unwrap();
        let right = chunk.push_constant(Value::integer(23)).unwrap();

        chunk.push_code(Instruction::Constant as u8, Span(0, 1));
        chunk.push_code(left, Span(2, 3));
        chunk.push_code(Instruction::Constant as u8, Span(4, 5));
        chunk.push_code(right, Span(6, 7));
        chunk.push_code(Instruction::Subtract as u8, Span(8, 9));
        chunk.push_code(Instruction::Return as u8, Span(10, 11));

        let mut vm = Vm::new(chunk);
        let result = vm.run();

        assert_eq!(result, Ok(Some(Value::integer(19))));
    }

    #[test]
    fn multiplication() {
        let mut chunk = Chunk::new();
        let left = chunk.push_constant(Value::integer(42)).unwrap();
        let right = chunk.push_constant(Value::integer(23)).unwrap();

        chunk.push_code(Instruction::Constant as u8, Span(0, 1));
        chunk.push_code(left, Span(2, 3));
        chunk.push_code(Instruction::Constant as u8, Span(4, 5));
        chunk.push_code(right, Span(6, 7));
        chunk.push_code(Instruction::Multiply as u8, Span(8, 9));
        chunk.push_code(Instruction::Return as u8, Span(10, 11));

        let mut vm = Vm::new(chunk);
        let result = vm.run();

        assert_eq!(result, Ok(Some(Value::integer(966))));
    }

    #[test]

    fn division() {
        let mut chunk = Chunk::new();
        let left = chunk.push_constant(Value::integer(42)).unwrap();
        let right = chunk.push_constant(Value::integer(23)).unwrap();

        chunk.push_code(Instruction::Constant as u8, Span(0, 1));
        chunk.push_code(left, Span(2, 3));
        chunk.push_code(Instruction::Constant as u8, Span(4, 5));
        chunk.push_code(right, Span(6, 7));
        chunk.push_code(Instruction::Divide as u8, Span(8, 9));
        chunk.push_code(Instruction::Return as u8, Span(10, 11));

        let mut vm = Vm::new(chunk);
        let result = vm.run();

        assert_eq!(result, Ok(Some(Value::integer(1))));
    }
}
