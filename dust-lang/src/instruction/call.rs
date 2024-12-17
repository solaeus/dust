use crate::{Instruction, Operation};

use super::InstructionData;

pub struct Call {
    pub destination: u8,
    pub function_register: u8,
    pub argument_count: u8,
}

impl From<&Instruction> for Call {
    fn from(instruction: &Instruction) -> Self {
        let destination = instruction.a_field();
        let function_register = instruction.b_field();
        let argument_count = instruction.c_field();

        Call {
            destination,
            function_register,
            argument_count,
        }
    }
}

impl From<InstructionData> for Call {
    fn from(instruction: InstructionData) -> Self {
        let destination = instruction.a_field;
        let function_register = instruction.b_field;
        let argument_count = instruction.c_field;

        Call {
            destination,
            function_register,
            argument_count,
        }
    }
}

impl From<Call> for Instruction {
    fn from(call: Call) -> Self {
        let a = call.destination;
        let b = call.function_register;
        let c = call.argument_count;

        Instruction::new(Operation::CALL, a, b, c, false, false, false)
    }
}
