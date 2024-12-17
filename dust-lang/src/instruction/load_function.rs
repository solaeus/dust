use std::fmt::{self, Display, Formatter};

use super::{Instruction, Operation};

pub struct LoadFunction {
    pub destination: u8,
    pub prototype_index: u8,
}

impl From<&Instruction> for LoadFunction {
    fn from(instruction: &Instruction) -> Self {
        let destination = instruction.a_field();
        let prototype_index = instruction.b_field();

        LoadFunction {
            destination,
            prototype_index,
        }
    }
}

impl From<LoadFunction> for Instruction {
    fn from(load_function: LoadFunction) -> Self {
        let operation = Operation::LOAD_FUNCTION;

        Instruction::new(
            operation,
            load_function.destination,
            load_function.prototype_index,
            0,
            false,
            false,
            false,
        )
    }
}

impl Display for LoadFunction {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "R{} = P{}", self.destination, self.prototype_index)
    }
}
