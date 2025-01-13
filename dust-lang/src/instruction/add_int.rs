use std::fmt::{self, Display, Formatter};

use crate::{Instruction, Operand, Operation};

use super::InstructionBuilder;

pub struct AddInt {
    pub destination: u16,
    pub left: Operand,
    pub right: Operand,
}

impl From<Instruction> for AddInt {
    fn from(instruction: Instruction) -> Self {
        let destination = instruction.a_field();
        let (left, right) = instruction.b_and_c_as_operands();

        AddInt {
            destination,
            left,
            right,
        }
    }
}

impl From<AddInt> for Instruction {
    fn from(add_int: AddInt) -> Self {
        let operation = Operation::ADD_INT;
        let a_field = add_int.destination;
        let (b_field, b_is_constant) = add_int.left.as_index_and_constant_flag();
        let (c_field, c_is_constant) = add_int.right.as_index_and_constant_flag();

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

impl Display for AddInt {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let AddInt {
            destination,
            left,
            right,
        } = self;

        write!(f, "R{} = {} + {}", destination, left, right)
    }
}
