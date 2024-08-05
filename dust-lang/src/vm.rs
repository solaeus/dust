use crate::{parse, Node, ParseError, Parser, Statement, Value, ValueError};

pub fn run(input: &str) -> Result<Option<Value>, VmError> {
    let instructions = parse(input)?;
    let vm = Vm::new(instructions);

    vm.run()
}

pub struct Vm {
    instructions: Vec<Node>,
}

impl Vm {
    pub fn new(instructions: Vec<Node>) -> Self {
        Vm { instructions }
    }

    pub fn run(&self) -> Result<Option<Value>, VmError> {
        let mut previous_value = None;

        for instruction in &self.instructions {
            previous_value = self.run_instruction(instruction)?;
        }

        Ok(previous_value)
    }

    fn run_instruction(&self, instruction: &Node) -> Result<Option<Value>, VmError> {
        match &instruction.operation {
            Statement::Add(instructions) => {
                let left = if let Some(value) = self.run_instruction(&instructions.0)? {
                    value
                } else {
                    return Err(VmError::ExpectedValue(instructions.0.operation.clone()));
                };
                let right = if let Some(value) = self.run_instruction(&instructions.1)? {
                    value
                } else {
                    return Err(VmError::ExpectedValue(instructions.1.operation.clone()));
                };
                let sum = left.add(&right)?;

                Ok(Some(sum))
            }
            Statement::Assign(_) => todo!(),
            Statement::Constant(value) => Ok(Some(value.clone())),
            Statement::Identifier(_) => todo!(),
            Statement::List(_) => todo!(),
            Statement::Multiply(_) => todo!(),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum VmError {
    ExpectedValue(Statement),
    InvalidOperation(Statement),
    ParseError(ParseError),
    ValueError(ValueError),
}

impl From<ParseError> for VmError {
    fn from(v: ParseError) -> Self {
        Self::ParseError(v)
    }
}

impl From<ValueError> for VmError {
    fn from(v: ValueError) -> Self {
        Self::ValueError(v)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn add() {
        let input = "1 + 2";

        assert_eq!(run(input), Ok(Some(Value::integer(3))));
    }

    #[test]
    fn add_multiple() {
        let input = "(a + b = 1)";

        assert_eq!(run(input), Ok(Some(Value::integer(6))));
    }
}
