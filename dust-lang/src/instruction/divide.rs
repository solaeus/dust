use std::fmt::{self, Display, Formatter};

use super::{Instruction, InstructionBuilder, Operand, Operation, TypeCode};

pub struct Divide {
    pub destination: u16,
    pub left: Operand,
    pub left_type: TypeCode,
    pub right: Operand,
    pub right_type: TypeCode,
}

impl From<Instruction> for Divide {
    fn from(instruction: Instruction) -> Self {
        let destination = instruction.a_field();
        let (left, right) = instruction.b_and_c_as_operands();
        let left_type = instruction.b_type();
        let right_type = instruction.c_type();

        Divide {
            destination,
            left,
            left_type,
            right,
            right_type,
        }
    }
}

impl From<Divide> for Instruction {
    fn from(divide: Divide) -> Self {
        let operation = Operation::DIVIDE;
        let a_field = divide.destination;
        let (b_field, b_is_constant) = divide.left.as_index_and_constant_flag();
        let (c_field, c_is_constant) = divide.right.as_index_and_constant_flag();
        let b_type = divide.left_type;
        let c_type = divide.right_type;

        InstructionBuilder {
            operation,
            a_field,
            b_field,
            c_field,
            b_is_constant,
            c_is_constant,
            b_type,
            c_type,
            ..Default::default()
        }
        .build()
    }
}

impl Display for Divide {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let Divide {
            destination,
            left,
            left_type: _,
            right,
            right_type: _,
        } = self;

        write!(f, "R{destination} = {left} ÷ {right}",)
    }
}
