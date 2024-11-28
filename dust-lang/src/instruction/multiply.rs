use crate::{Argument, Destination, Instruction, Operation};

pub struct Multiply {
    pub destination: Destination,
    pub left: Argument,
    pub right: Argument,
}

impl From<&Instruction> for Multiply {
    fn from(instruction: &Instruction) -> Self {
        let destination = if instruction.a_is_local() {
            Destination::Local(instruction.a())
        } else {
            Destination::Register(instruction.a())
        };
        let (left, right) = instruction.b_and_c_as_arguments();

        Multiply {
            destination,
            left,
            right,
        }
    }
}

impl From<Multiply> for Instruction {
    fn from(multiply: Multiply) -> Self {
        let (a, a_is_local) = match multiply.destination {
            Destination::Local(local) => (local, true),
            Destination::Register(register) => (register, false),
        };

        *Instruction::new(Operation::Multiply)
            .set_a(a)
            .set_a_is_local(a_is_local)
            .set_b(multiply.left.index())
            .set_b_is_constant(multiply.left.is_constant())
            .set_b_is_local(multiply.left.is_local())
            .set_c(multiply.right.index())
            .set_c_is_constant(multiply.right.is_constant())
            .set_c_is_local(multiply.right.is_local())
    }
}
