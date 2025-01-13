use std::fmt::{self, Display, Formatter};

use super::{Instruction, InstructionBuilder, Operand, Operation, TypeCode};

pub struct Negate {
    pub destination: u16,
    pub argument: Operand,
    pub argument_type: TypeCode,
}

impl From<Instruction> for Negate {
    fn from(instruction: Instruction) -> Self {
        let destination = instruction.a_field();
        let argument = instruction.b_as_argument();
        let argument_type = instruction.b_type();

        Negate {
            destination,
            argument,
            argument_type,
        }
    }
}

impl From<Negate> for Instruction {
    fn from(negate: Negate) -> Self {
        let operation = Operation::NEGATE;
        let a_field = negate.destination;
        let (b_field, b_is_constant) = negate.argument.as_index_and_constant_flag();
        let b_type = negate.argument_type;

        InstructionBuilder {
            operation,
            a_field,
            b_field,
            b_is_constant,
            b_type,
            ..Default::default()
        }
        .build()
    }
}

impl Display for Negate {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let Negate {
            destination,
            argument,
            argument_type,
        } = self;

        write!(f, "R{destination} = -{argument_type}({argument})")
    }
}
