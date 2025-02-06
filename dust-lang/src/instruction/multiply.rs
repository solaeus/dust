use std::fmt::{self, Display, Formatter};

use super::{Instruction, InstructionFields, Operand, Operation, TypeCode};

pub struct Multiply {
    pub destination: u16,
    pub left: Operand,
    pub left_type: TypeCode,
    pub right: Operand,
    pub right_type: TypeCode,
}

impl From<Instruction> for Multiply {
    fn from(instruction: Instruction) -> Self {
        let destination = instruction.a_field();
        let (left, right) = instruction.b_and_c_as_operands();
        let left_type = instruction.b_type();
        let right_type = instruction.c_type();

        Multiply {
            destination,
            left,
            left_type,
            right,
            right_type,
        }
    }
}

impl From<Multiply> for Instruction {
    fn from(multiply: Multiply) -> Self {
        let operation = Operation::MULTIPLY;
        let a_field = multiply.destination;
        let (b_field, b_is_constant) = multiply.left.as_index_and_constant_flag();
        let (c_field, c_is_constant) = multiply.right.as_index_and_constant_flag();
        let b_type = multiply.left_type;
        let c_type = multiply.right_type;

        InstructionFields {
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

impl Display for Multiply {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let Multiply {
            destination,
            left,
            left_type: _,
            right,
            right_type: _,
        } = self;

        write!(f, "R{destination} = {left} ✕ {right}",)
    }
}
