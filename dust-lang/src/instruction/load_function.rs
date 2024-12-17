use std::fmt::{self, Display, Formatter};

use super::{Instruction, InstructionData, Operation};

pub struct LoadFunction {
    pub destination: u8,
    pub record_index: u8,
}

impl From<&Instruction> for LoadFunction {
    fn from(instruction: &Instruction) -> Self {
        let destination = instruction.a_field();
        let record_index = instruction.b_field();

        LoadFunction {
            destination,
            record_index,
        }
    }
}

impl From<InstructionData> for LoadFunction {
    fn from(instruction: InstructionData) -> Self {
        LoadFunction {
            destination: instruction.a_field,
            record_index: instruction.b_field,
        }
    }
}

impl From<LoadFunction> for Instruction {
    fn from(load_function: LoadFunction) -> Self {
        Instruction::new(
            Operation::LOAD_FUNCTION,
            load_function.destination,
            load_function.record_index,
            0,
            false,
            false,
            false,
        )
    }
}

impl Display for LoadFunction {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "R{} = P{}", self.destination, self.record_index)
    }
}
