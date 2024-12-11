use crate::{Argument, Instruction, Operation};

pub struct Call {
    pub destination: u8,
    pub function: Argument,
    pub argument_count: u8,
}

impl From<&Instruction> for Call {
    fn from(instruction: &Instruction) -> Self {
        let destination = instruction.a_field();
        let function = instruction.b_as_argument();
        let argument_count = instruction.c_field();

        Call {
            destination,
            function,
            argument_count,
        }
    }
}

impl From<Call> for Instruction {
    fn from(call: Call) -> Self {
        let a = call.destination;
        let (b, b_is_constant) = call.function.as_index_and_constant_flag();
        let c = call.argument_count;

        Instruction::new(Operation::CALL, a, b, c, b_is_constant, false, false)
    }
}
