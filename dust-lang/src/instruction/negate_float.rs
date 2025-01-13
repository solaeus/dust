use std::fmt::{self, Display, Formatter};

use super::{Instruction, InstructionBuilder, Operand, Operation};

pub struct NegateFloat {
    pub destination: u16,
    pub argument: Operand,
}

impl From<Instruction> for NegateFloat {
    fn from(instruction: Instruction) -> Self {
        let destination = instruction.a_field();
        let argument = instruction.b_as_argument();

        NegateFloat {
            destination,
            argument,
        }
    }
}

impl From<NegateFloat> for Instruction {
    fn from(negate_float: NegateFloat) -> Self {
        let operation = Operation::NEGATE_FLOAT;
        let a_field = negate_float.destination;
        let (b_field, b_is_constant) = negate_float.argument.as_index_and_constant_flag();

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

impl Display for NegateFloat {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let NegateFloat {
            destination,
            argument,
        } = self;

        write!(f, "R{destination} = -{argument}")
    }
}
