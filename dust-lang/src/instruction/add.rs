use crate::{Argument, Destination, Instruction, Operation};

pub struct Add {
    pub destination: Destination,
    pub left: Argument,
    pub right: Argument,
}

impl From<&Instruction> for Add {
    fn from(instruction: &Instruction) -> Self {
        let destination = if instruction.a_is_local() {
            Destination::Local(instruction.a())
        } else {
            Destination::Register(instruction.a())
        };
        let (left, right) = instruction.b_and_c_as_arguments();

        Add {
            destination,
            left,
            right,
        }
    }
}

impl From<Add> for Instruction {
    fn from(add: Add) -> Self {
        let (a, a_is_local) = match add.destination {
            Destination::Local(local) => (local, true),
            Destination::Register(register) => (register, false),
        };

        *Instruction::new(Operation::Add)
            .set_a(a)
            .set_a_is_local(a_is_local)
            .set_b(add.left.index())
            .set_b_is_constant(add.left.is_constant())
            .set_b_is_local(add.left.is_local())
            .set_c(add.right.index())
            .set_c_is_constant(add.right.is_constant())
            .set_c_is_local(add.right.is_local())
    }
}
