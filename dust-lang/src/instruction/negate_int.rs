use std::fmt::{self, Display, Formatter};

use super::{Instruction, InstructionBuilder, Operand, Operation};

pub struct NegateInt {
    pub destination: u16,
    pub argument: Operand,
}

impl From<Instruction> for NegateInt {
    fn from(instruction: Instruction) -> Self {
        let destination = instruction.a_field();
        let argument = instruction.b_as_argument();

        NegateInt {
            destination,
            argument,
        }
    }
}

impl From<NegateInt> for Instruction {
    fn from(negate_int: NegateInt) -> Self {
        let operation = Operation::NEGATE_INT;
        let a_field = negate_int.destination;
        let (b_field, b_is_constant) = negate_int.argument.as_index_and_constant_flag();

        InstructionBuilder {
            operation,
            a_field,
            b_field,
            b_is_constant,
            ..Default::default()
        }
        .build()
    }
}

impl Display for NegateInt {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let NegateInt {
            destination,
            argument,
        } = self;

        write!(f, "R{destination} = -{argument}")
    }
}
