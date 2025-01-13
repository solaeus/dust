use std::fmt::{self, Display, Formatter};

use super::{Instruction, InstructionBuilder, Operand, Operation};

pub struct MultiplyByte {
    pub destination: u16,
    pub left: Operand,
    pub right: Operand,
}

impl From<Instruction> for MultiplyByte {
    fn from(instruction: Instruction) -> Self {
        let destination = instruction.a_field();
        let (left, right) = instruction.b_and_c_as_operands();

        MultiplyByte {
            destination,
            left,
            right,
        }
    }
}

impl From<MultiplyByte> for Instruction {
    fn from(multiply_byte: MultiplyByte) -> Self {
        let operation = Operation::MODULO_BYTE;
        let a_field = multiply_byte.destination;
        let (b_field, b_is_constant) = multiply_byte.left.as_index_and_constant_flag();
        let (c_field, c_is_constant) = multiply_byte.right.as_index_and_constant_flag();

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

impl Display for MultiplyByte {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let MultiplyByte {
            destination,
            left,
            right,
        } = self;

        write!(f, "R{} = {} ✕ {}", destination, left, right)
    }
}
