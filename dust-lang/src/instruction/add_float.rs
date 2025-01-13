use std::fmt::{self, Display, Formatter};

use super::{Instruction, InstructionBuilder, Operand, Operation};

pub struct AddFloat {
    pub destination: u16,
    pub left: Operand,
    pub right: Operand,
}

impl From<Instruction> for AddFloat {
    fn from(instruction: Instruction) -> Self {
        let destination = instruction.a_field();
        let (left, right) = instruction.b_and_c_as_operands();

        AddFloat {
            destination,
            left,
            right,
        }
    }
}

impl From<AddFloat> for Instruction {
    fn from(add_float: AddFloat) -> Self {
        let operation = Operation::ADD_FLOAT;
        let a_field = add_float.destination;
        let (b_field, b_is_constant) = add_float.left.as_index_and_constant_flag();
        let (c_field, c_is_constant) = add_float.right.as_index_and_constant_flag();

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

impl Display for AddFloat {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let AddFloat {
            destination,
            left,
            right,
        } = self;

        write!(f, "R{} = {} + {}", destination, left, right)
    }
}
