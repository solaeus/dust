use crate::{
    parse, Chunk, ChunkError, DustError, Identifier, Instruction, Span, Value, ValueError,
};

pub fn run(source: &str) -> Result<Option<Value>, DustError> {
    let chunk = parse(source)?;

    let mut vm = Vm::new(chunk);

    vm.run()
        .map_err(|error| DustError::Runtime { error, source })
}

#[derive(Debug, Clone, Eq, PartialEq)]
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
        let mut current_postion = Span(0, 0);

        while let Ok((byte, position)) = self.read(current_postion).copied() {
            current_postion = position;

            let instruction = Instruction::from_byte(byte)
                .ok_or_else(|| VmError::InvalidInstruction(byte, position))?;

            log::trace!("Running instruction {instruction} at {position}");

            match instruction {
                Instruction::Constant => {
                    let (index, _) = self.read(position).copied()?;
                    let value = self.read_constant(index, position)?.clone();

                    self.push(value, position)?;
                }
                Instruction::Return => {
                    let value = self.pop(position)?;

                    return Ok(Some(value));
                }
                Instruction::Pop => {
                    self.pop(position)?;
                }

                // Variables
                Instruction::DefineVariable => {
                    let (index, _) = *self.read(position)?;
                    let value = self
                        .read_constant(index, position)?
                        .clone()
                        .into_reference();

                    self.stack.insert(index as usize, value);
                }
                Instruction::GetVariable => {
                    let (index, _) = *self.read(position)?;
                    let value = self.stack[index as usize].clone();

                    self.push(value, position)?;
                }
                Instruction::SetVariable => {
                    let (index, _) = *self.read(position)?;
                    let identifier = self
                        .chunk
                        .get_identifier(index)
                        .map_err(|error| VmError::Chunk { error, position })?
                        .clone();

                    if !self.chunk.contains_identifier(&identifier) {
                        return Err(VmError::UndefinedVariable(identifier, position));
                    }

                    let value = self.pop(position)?;

                    self.stack[index as usize] = value;
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

        Ok(self.stack.pop())
    }

    fn push(&mut self, value: Value, position: Span) -> Result<(), VmError> {
        if self.stack.len() == Self::STACK_SIZE {
            Err(VmError::StackOverflow(position))
        } else {
            self.stack.push(value);

            Ok(())
        }
    }

    fn pop(&mut self, position: Span) -> Result<Value, VmError> {
        if let Some(value) = self.stack.pop() {
            Ok(value)
        } else {
            Err(VmError::StackUnderflow(position))
        }
    }

    fn read(&mut self, position: Span) -> Result<&(u8, Span), VmError> {
        let current = self
            .chunk
            .get_code(self.ip)
            .map_err(|error| VmError::Chunk { error, position })?;

        self.ip += 1;

        Ok(current)
    }

    fn read_constant(&self, index: u8, position: Span) -> Result<&Value, VmError> {
        let value = self
            .chunk
            .get_constant(index)
            .map_err(|error| VmError::Chunk { error, position })?;

        Ok(value)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum VmError {
    InvalidInstruction(u8, Span),
    StackOverflow(Span),
    StackUnderflow(Span),
    UndefinedVariable(Identifier, Span),

    // Wrappers for foreign errors
    Chunk { error: ChunkError, position: Span },
    Value { error: ValueError, position: Span },
}

impl VmError {
    pub fn chunk(error: ChunkError, position: Span) -> Self {
        Self::Chunk { error, position }
    }

    pub fn value(error: ValueError, position: Span) -> Self {
        Self::Value { error, position }
    }

    pub fn title(&self) -> &'static str {
        "VM Error"
    }

    pub fn description(&self) -> String {
        match self {
            Self::InvalidInstruction(byte, _) => {
                format!("The byte {byte} does not correspond to a valid instruction")
            }
            Self::StackOverflow(position) => format!("Stack overflow at {position}"),
            Self::StackUnderflow(position) => format!("Stack underflow at {position}"),
            Self::UndefinedVariable(identifier, position) => {
                format!("{identifier} is not in scope at {position}")
            }

            Self::Chunk { error, .. } => error.description(),
            Self::Value { error, .. } => error.description(),
        }
    }

    pub fn position(&self) -> Span {
        match self {
            Self::InvalidInstruction(_, position) => *position,
            Self::StackUnderflow(position) => *position,
            Self::StackOverflow(position) => *position,
            Self::UndefinedVariable(_, position) => *position,
            Self::Chunk { position, .. } => *position,
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
