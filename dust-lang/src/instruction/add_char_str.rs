use std::fmt::{self, Display, Formatter};

use super::{Instruction, InstructionBuilder, Operand, Operation};

pub struct AddCharStr {
    pub destination: u16,
    pub left: Operand,
    pub right: Operand,
}

impl From<Instruction> for AddCharStr {
    fn from(instruction: Instruction) -> Self {
        let destination = instruction.a_field();
        let (left, right) = instruction.b_and_c_as_operands();

        AddCharStr {
            destination,
            left,
            right,
        }
    }
}

impl From<AddCharStr> for Instruction {
    fn from(add_char_str: AddCharStr) -> Self {
        let operation = Operation::ADD_CHAR_STR;
        let a_field = add_char_str.destination;
        let (b_field, b_is_constant) = add_char_str.left.as_index_and_constant_flag();
        let (c_field, c_is_constant) = add_char_str.right.as_index_and_constant_flag();

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

impl Display for AddCharStr {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let AddCharStr {
            destination,
            left,
            right,
        } = self;

        write!(f, "R{} = {} + {}", destination, left, right)
    }
}
