use crate::{Argument, Destination, Instruction, Operation};

pub struct Negate {
    pub destination: Destination,
    pub argument: Argument,
}

impl From<&Instruction> for Negate {
    fn from(instruction: &Instruction) -> Self {
        let destination = if instruction.a_is_local() {
            Destination::Local(instruction.a())
        } else {
            Destination::Register(instruction.a())
        };

        Negate {
            destination,
            argument: instruction.b_as_argument(),
        }
    }
}

impl From<Negate> for Instruction {
    fn from(negate: Negate) -> Self {
        let (a, a_is_local) = match negate.destination {
            Destination::Local(local) => (local, true),
            Destination::Register(register) => (register, false),
        };

        *Instruction::new(Operation::Negate)
            .set_a(a)
            .set_a_is_local(a_is_local)
            .set_b(negate.argument.index())
            .set_b_is_constant(negate.argument.is_constant())
            .set_b_is_local(negate.argument.is_local())
    }
}
