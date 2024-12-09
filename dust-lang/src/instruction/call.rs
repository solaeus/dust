use crate::{Argument, Instruction, Operation};

pub struct Call {
    pub destination: u8,
    pub function: Argument,
    pub argument_count: u8,
}

impl From<&Instruction> for Call {
    fn from(instruction: &Instruction) -> Self {
        let destination = instruction.a;
        let function = instruction.b_as_argument();
        let argument_count = instruction.c;

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
        let (b, b_options) = call.function.as_index_and_b_options();
        let c = call.argument_count;
        let metadata = Operation::Call as u8 | b_options.bits();

        Instruction { metadata, a, b, c }
    }
}
