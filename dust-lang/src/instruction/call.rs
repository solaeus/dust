use crate::{Argument, Instruction, Operation};

pub struct Call {
    pub destination: u16,
    pub function: Argument,
    pub argument_count: u16,
}

impl From<&Instruction> for Call {
    fn from(instruction: &Instruction) -> Self {
        Call {
            destination: instruction.a(),
            function: instruction.b_as_argument(),
            argument_count: instruction.c(),
        }
    }
}

impl From<Call> for Instruction {
    fn from(call: Call) -> Self {
        *Instruction::new(Operation::Call)
            .set_a(call.destination)
            .set_b(call.function.index())
            .set_b_is_constant(call.function.is_constant())
            .set_b_is_local(call.function.is_local())
            .set_c(call.argument_count)
    }
}
