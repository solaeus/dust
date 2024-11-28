use crate::{Argument, Destination, Instruction, Operation};

pub struct Divide {
    pub destination: Destination,
    pub left: Argument,
    pub right: Argument,
}

impl From<&Instruction> for Divide {
    fn from(instruction: &Instruction) -> Self {
        let destination = if instruction.a_is_local() {
            Destination::Local(instruction.a())
        } else {
            Destination::Register(instruction.a())
        };
        let (left, right) = instruction.b_and_c_as_arguments();

        Divide {
            destination,
            left,
            right,
        }
    }
}

impl From<Divide> for Instruction {
    fn from(divide: Divide) -> Self {
        let (a, a_is_local) = match divide.destination {
            Destination::Local(local) => (local, true),
            Destination::Register(register) => (register, false),
        };

        *Instruction::new(Operation::Divide)
            .set_a(a)
            .set_a_is_local(a_is_local)
            .set_b(divide.left.index())
            .set_b_is_constant(divide.left.is_constant())
            .set_b_is_local(divide.left.is_local())
            .set_c(divide.right.index())
            .set_c_is_constant(divide.right.is_constant())
            .set_c_is_local(divide.right.is_local())
    }
}
