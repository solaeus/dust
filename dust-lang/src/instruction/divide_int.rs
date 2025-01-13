use std::fmt::{self, Display, Formatter};

use super::{Instruction, InstructionBuilder, Operand, Operation};

pub struct DivideInt {
    pub destination: u16,
    pub left: Operand,
    pub right: Operand,
}

impl From<Instruction> for DivideInt {
    fn from(instruction: Instruction) -> Self {
        let destination = instruction.a_field();
        let (left, right) = instruction.b_and_c_as_operands();

        DivideInt {
            destination,
            left,
            right,
        }
    }
}

impl From<DivideInt> for Instruction {
    fn from(divide_int: DivideInt) -> Self {
        let operation = Operation::DIVIDE_INT;
        let a_field = divide_int.destination;
        let (b_field, b_is_constant) = divide_int.left.as_index_and_constant_flag();
        let (c_field, c_is_constant) = divide_int.right.as_index_and_constant_flag();

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

impl Display for DivideInt {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let DivideInt {
            destination,
            left,
            right,
        } = self;

        write!(f, "R{} = {} ÷ {}", destination, left, right)
    }
}
