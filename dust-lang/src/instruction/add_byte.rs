use std::fmt::Display;

use super::{Instruction, InstructionBuilder, Operand, Operation};

pub struct AddByte {
    pub destination: u16,
    pub left: Operand,
    pub right: Operand,
}

impl From<Instruction> for AddByte {
    fn from(instruction: Instruction) -> Self {
        let destination = instruction.a_field();
        let (left, right) = instruction.b_and_c_as_operands();

        AddByte {
            destination,
            left,
            right,
        }
    }
}

impl From<AddByte> for Instruction {
    fn from(add_byte: AddByte) -> Self {
        let operation = Operation::ADD_BYTE;
        let a_field = add_byte.destination;
        let (b_field, b_is_constant) = add_byte.left.as_index_and_constant_flag();
        let (c_field, c_is_constant) = add_byte.right.as_index_and_constant_flag();

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

impl Display for AddByte {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let AddByte {
            destination,
            left,
            right,
        } = self;

        write!(f, "R{} = {} + {}", destination, left, right)
    }
}
