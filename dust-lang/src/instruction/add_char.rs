use std::fmt::{self, Display, Formatter};

use super::{Instruction, InstructionBuilder, Operand, Operation};

pub struct AddChar {
    pub destination: u16,
    pub left: Operand,
    pub right: Operand,
}

impl From<Instruction> for AddChar {
    fn from(instruction: Instruction) -> Self {
        let destination = instruction.a_field();
        let (left, right) = instruction.b_and_c_as_operands();

        AddChar {
            destination,
            left,
            right,
        }
    }
}

impl From<AddChar> for Instruction {
    fn from(add_char: AddChar) -> Self {
        let operation = Operation::ADD_CHAR;
        let a_field = add_char.destination;
        let (b_field, b_is_constant) = add_char.left.as_index_and_constant_flag();
        let (c_field, c_is_constant) = add_char.right.as_index_and_constant_flag();

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

impl Display for AddChar {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let AddChar {
            destination,
            left,
            right,
        } = self;

        write!(f, "R{} = {} + {}", destination, left, right)
    }
}
