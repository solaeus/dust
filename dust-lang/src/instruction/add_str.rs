use std::fmt::{self, Display, Formatter};

use super::{Instruction, InstructionBuilder, Operand, Operation};

pub struct AddStr {
    pub destination: u16,
    pub left: Operand,
    pub right: Operand,
}

impl From<Instruction> for AddStr {
    fn from(instruction: Instruction) -> Self {
        let destination = instruction.a_field();
        let (left, right) = instruction.b_and_c_as_operands();

        AddStr {
            destination,
            left,
            right,
        }
    }
}

impl From<AddStr> for Instruction {
    fn from(add_str: AddStr) -> Self {
        let operation = Operation::ADD_STR;
        let a_field = add_str.destination;
        let (b_field, b_is_constant) = add_str.left.as_index_and_constant_flag();
        let (c_field, c_is_constant) = add_str.right.as_index_and_constant_flag();

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

impl Display for AddStr {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let AddStr {
            destination,
            left,
            right,
        } = self;

        write!(f, "R{} = {} + {}", destination, left, right)
    }
}
