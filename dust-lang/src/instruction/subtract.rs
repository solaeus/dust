use crate::{Argument, Destination, Instruction};

pub struct Subtract {
    pub destination: Destination,
    pub left: Argument,
    pub right: Argument,
}

impl From<&Instruction> for Subtract {
    fn from(instruction: &Instruction) -> Self {
        let destination = if instruction.a_is_local() {
            Destination::Local(instruction.a())
        } else {
            Destination::Register(instruction.a())
        };
        let (left, right) = instruction.b_and_c_as_arguments();

        Subtract {
            destination,
            left,
            right,
        }
    }
}

impl From<Subtract> for Instruction {
    fn from(subtract: Subtract) -> Self {
        let (a, a_is_local) = match subtract.destination {
            Destination::Local(local) => (local, true),
            Destination::Register(register) => (register, false),
        };

        *Instruction::new(crate::Operation::Subtract)
            .set_a(a)
            .set_a_is_local(a_is_local)
            .set_b(subtract.left.index())
            .set_b_is_constant(subtract.left.is_constant())
            .set_b_is_local(subtract.left.is_local())
            .set_c(subtract.right.index())
            .set_c_is_constant(subtract.right.is_constant())
            .set_c_is_local(subtract.right.is_local())
    }
}
