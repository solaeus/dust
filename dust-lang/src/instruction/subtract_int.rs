use std::fmt::{self, Display, Formatter};

use super::{Instruction, InstructionBuilder, Operand, Operation};

pub struct SubtractInt {
    pub destination: u16,
    pub left: Operand,
    pub right: Operand,
}

impl From<Instruction> for SubtractInt {
    fn from(instruction: Instruction) -> Self {
        let destination = instruction.a_field();
        let (left, right) = instruction.b_and_c_as_operands();

        SubtractInt {
            destination,
            left,
            right,
        }
    }
}

impl From<SubtractInt> for Instruction {
    fn from(subtract_int: SubtractInt) -> Self {
        let operation = Operation::SUBTRACT_INT;
        let a_field = subtract_int.destination;
        let (b_field, b_is_constant) = subtract_int.left.as_index_and_constant_flag();
        let (c_field, c_is_constant) = subtract_int.right.as_index_and_constant_flag();

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

impl Display for SubtractInt {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let SubtractInt {
            destination,
            left,
            right,
        } = self;

        write!(f, "R{} = {} - {}", destination, left, right)
    }
}
