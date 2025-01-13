use std::fmt::{self, Display, Formatter};

use super::{Instruction, InstructionBuilder, Operand, Operation};

pub struct MultiplyInt {
    pub destination: u16,
    pub left: Operand,
    pub right: Operand,
}

impl From<Instruction> for MultiplyInt {
    fn from(instruction: Instruction) -> Self {
        let destination = instruction.a_field();
        let (left, right) = instruction.b_and_c_as_operands();

        MultiplyInt {
            destination,
            left,
            right,
        }
    }
}

impl From<MultiplyInt> for Instruction {
    fn from(multiply_int: MultiplyInt) -> Self {
        let operation = Operation::MULTIPLY_INT;
        let a_field = multiply_int.destination;
        let (b_field, b_is_constant) = multiply_int.left.as_index_and_constant_flag();
        let (c_field, c_is_constant) = multiply_int.right.as_index_and_constant_flag();

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

impl Display for MultiplyInt {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let MultiplyInt {
            destination,
            left,
            right,
        } = self;

        write!(f, "R{} = {} ✕ {}", destination, left, right)
    }
}
