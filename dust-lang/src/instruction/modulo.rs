use crate::{Argument, Destination, Instruction, Operation};

pub struct Modulo {
    pub destination: Destination,
    pub left: Argument,
    pub right: Argument,
}

impl From<&Instruction> for Modulo {
    fn from(instruction: &Instruction) -> Self {
        let destination = if instruction.a_is_local() {
            Destination::Local(instruction.a())
        } else {
            Destination::Register(instruction.a())
        };
        let (left, right) = instruction.b_and_c_as_arguments();

        Modulo {
            destination,
            left,
            right,
        }
    }
}

impl From<Modulo> for Instruction {
    fn from(modulo: Modulo) -> Self {
        let (a, a_is_local) = match modulo.destination {
            Destination::Local(local) => (local, true),
            Destination::Register(register) => (register, false),
        };

        *Instruction::new(Operation::Modulo)
            .set_a(a)
            .set_a_is_local(a_is_local)
            .set_b(modulo.left.index())
            .set_b_is_constant(modulo.left.is_constant())
            .set_b_is_local(modulo.left.is_local())
            .set_c(modulo.right.index())
            .set_c_is_constant(modulo.right.is_constant())
            .set_c_is_local(modulo.right.is_local())
    }
}
