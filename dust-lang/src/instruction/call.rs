use crate::{Argument, Destination, Instruction, Operation};

pub struct Call {
    pub destination: Destination,
    pub function: Argument,
    pub argument_count: u16,
}

impl From<&Instruction> for Call {
    fn from(instruction: &Instruction) -> Self {
        let destination = if instruction.a_is_local() {
            Destination::Local(instruction.a())
        } else {
            Destination::Register(instruction.a())
        };

        Call {
            destination,
            function: instruction.b_as_argument(),
            argument_count: instruction.c(),
        }
    }
}

impl From<Call> for Instruction {
    fn from(call: Call) -> Self {
        let (a, a_is_local) = match call.destination {
            Destination::Local(local) => (local, true),
            Destination::Register(register) => (register, false),
        };

        *Instruction::new(Operation::Call)
            .set_a(a)
            .set_a_is_local(a_is_local)
            .set_b(call.function.index())
            .set_b_is_constant(call.function.is_constant())
            .set_b_is_local(call.function.is_local())
            .set_c(call.argument_count)
    }
}
