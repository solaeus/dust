use crate::{Argument, Instruction, Operation};

pub struct Negate {
    pub destination: u16,
    pub argument: Argument,
}

impl From<&Instruction> for Negate {
    fn from(instruction: &Instruction) -> Self {
        Negate {
            destination: instruction.a(),
            argument: instruction.b_as_argument(),
        }
    }
}

impl From<Negate> for Instruction {
    fn from(negate: Negate) -> Self {
        *Instruction::new(Operation::Negate)
            .set_a(negate.destination)
            .set_b(negate.argument.index())
            .set_b_is_constant(negate.argument.is_constant())
            .set_b_is_local(negate.argument.is_local())
    }
}
