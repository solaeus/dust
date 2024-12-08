use crate::{Argument, Destination, Instruction, Operation};

pub struct Call {
    pub destination: Destination,
    pub function: Argument,
    pub argument_count: u16,
}

impl From<&Instruction> for Call {
    fn from(instruction: &Instruction) -> Self {
        Call {
            destination: instruction.a_as_destination(),
            function: instruction.b_as_argument(),
            argument_count: instruction.c,
        }
    }
}

impl From<Call> for Instruction {
    fn from(call: Call) -> Self {
        let (a, a_options) = call.destination.as_index_and_a_options();
        let (b, b_options) = call.function.as_index_and_b_options();

        Instruction {
            operation: Operation::CALL,
            options: a_options | b_options,
            a,
            b,
            c: call.argument_count,
        }
    }
}
