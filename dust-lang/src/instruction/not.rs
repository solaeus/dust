use crate::{Argument, Instruction, Operation};

pub struct Not {
    pub destination: u16,
    pub argument: Argument,
}

impl From<&Instruction> for Not {
    fn from(instruction: &Instruction) -> Self {
        Not {
            destination: instruction.a(),
            argument: instruction.b_as_argument(),
        }
    }
}

impl From<Not> for Instruction {
    fn from(not: Not) -> Self {
        *Instruction::new(Operation::Not)
            .set_a(not.destination)
            .set_b(not.argument.index())
            .set_b_is_constant(not.argument.is_constant())
            .set_b_is_local(not.argument.is_local())
    }
}
