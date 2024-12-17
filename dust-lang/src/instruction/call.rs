use crate::{Instruction, Operation};

pub struct Call {
    pub destination: u8,
    pub prototype_index: u8,
    pub argument_count: u8,
}

impl From<&Instruction> for Call {
    fn from(instruction: &Instruction) -> Self {
        let destination = instruction.a_field();
        let prototype_index = instruction.b_field();
        let argument_count = instruction.c_field();

        Call {
            destination,
            prototype_index,
            argument_count,
        }
    }
}

impl From<Call> for Instruction {
    fn from(call: Call) -> Self {
        let a = call.destination;
        let b = call.prototype_index;
        let c = call.argument_count;

        Instruction::new(Operation::CALL, a, b, c, false, false, false)
    }
}
