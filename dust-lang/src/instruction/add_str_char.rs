use std::fmt::{self, Display, Formatter};

use super::{Instruction, InstructionBuilder, Operand, Operation};

pub struct AddStrChar {
    pub destination: u16,
    pub left: Operand,
    pub right: Operand,
}

impl From<Instruction> for AddStrChar {
    fn from(instruction: Instruction) -> Self {
        let destination = instruction.a_field();
        let (left, right) = instruction.b_and_c_as_operands();

        AddStrChar {
            destination,
            left,
            right,
        }
    }
}

impl From<AddStrChar> for Instruction {
    fn from(add_str_char: AddStrChar) -> Self {
        let operation = Operation::ADD_STR_CHAR;
        let a_field = add_str_char.destination;
        let (b_field, b_is_constant) = add_str_char.left.as_index_and_constant_flag();
        let (c_field, c_is_constant) = add_str_char.right.as_index_and_constant_flag();

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

impl Display for AddStrChar {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let AddStrChar {
            destination,
            left,
            right,
        } = self;

        write!(f, "R{} = {} + {}", destination, left, right)
    }
}
