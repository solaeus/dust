use std::fmt::{self, Display, Formatter};

use super::{Instruction, InstructionBuilder, Operand, Operation};

pub struct DivideByte {
    pub destination: u16,
    pub left: Operand,
    pub right: Operand,
}

impl From<Instruction> for DivideByte {
    fn from(instruction: Instruction) -> Self {
        let destination = instruction.a_field();
        let (left, right) = instruction.b_and_c_as_operands();

        DivideByte {
            destination,
            left,
            right,
        }
    }
}

impl From<DivideByte> for Instruction {
    fn from(divide_byte: DivideByte) -> Self {
        let operation = Operation::DIVIDE_BYTE;
        let a_field = divide_byte.destination;
        let (b_field, b_is_constant) = divide_byte.left.as_index_and_constant_flag();
        let (c_field, c_is_constant) = divide_byte.right.as_index_and_constant_flag();

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

impl Display for DivideByte {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let DivideByte {
            destination,
            left,
            right,
        } = self;

        write!(f, "R{} = {} ÷ {}", destination, left, right)
    }
}
