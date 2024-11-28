use crate::{Argument, Destination, Instruction, Operation};

pub struct Not {
    pub destination: Destination,
    pub argument: Argument,
}

impl From<&Instruction> for Not {
    fn from(instruction: &Instruction) -> Self {
        let destination = if instruction.a_is_local() {
            Destination::Local(instruction.a())
        } else {
            Destination::Register(instruction.a())
        };

        Not {
            destination,
            argument: instruction.b_as_argument(),
        }
    }
}

impl From<Not> for Instruction {
    fn from(not: Not) -> Self {
        let (a, a_is_local) = match not.destination {
            Destination::Local(local) => (local, true),
            Destination::Register(register) => (register, false),
        };

        *Instruction::new(Operation::Not)
            .set_a(a)
            .set_a_is_local(a_is_local)
            .set_b(not.argument.index())
            .set_b_is_constant(not.argument.is_constant())
            .set_b_is_local(not.argument.is_local())
    }
}
