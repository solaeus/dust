use std::fmt::{self, Display, Formatter};

use super::{Instruction, InstructionBuilder, Operand, Operation};

pub struct SubtractByte {
    pub destination: u16,
    pub left: Operand,
    pub right: Operand,
}

impl From<Instruction> for SubtractByte {
    fn from(instruction: Instruction) -> Self {
        let destination = instruction.a_field();
        let (left, right) = instruction.b_and_c_as_operands();

        SubtractByte {
            destination,
            left,
            right,
        }
    }
}

impl From<SubtractByte> for Instruction {
    fn from(subtract_byte: SubtractByte) -> Self {
        let operation = Operation::SUBTRACT_BYTE;
        let a_field = subtract_byte.destination;
        let (b_field, b_is_constant) = subtract_byte.left.as_index_and_constant_flag();
        let (c_field, c_is_constant) = subtract_byte.right.as_index_and_constant_flag();

        InstructionBuilder {
            operation,
            a_field,
            b_field,
            c_field,
            b_is_constant,
            c_is_constant,
            ..Default::default()
        }
        .build()
    }
}

impl Display for SubtractByte {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let SubtractByte {
            destination,
            left,
            right,
        } = self;

        write!(f, "R{} = {} - {}", destination, left, right)
    }
}
