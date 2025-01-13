use std::fmt::{self, Display, Formatter};

use super::{Instruction, InstructionBuilder, Operand, Operation};

pub struct DivideFloat {
    pub destination: u16,
    pub left: Operand,
    pub right: Operand,
}

impl From<Instruction> for DivideFloat {
    fn from(instruction: Instruction) -> Self {
        let destination = instruction.a_field();
        let (left, right) = instruction.b_and_c_as_operands();

        DivideFloat {
            destination,
            left,
            right,
        }
    }
}

impl From<DivideFloat> for Instruction {
    fn from(divide_float: DivideFloat) -> Self {
        let operation = Operation::DIVIDE_FLOAT;
        let a_field = divide_float.destination;
        let (b_field, b_is_constant) = divide_float.left.as_index_and_constant_flag();
        let (c_field, c_is_constant) = divide_float.right.as_index_and_constant_flag();

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

impl Display for DivideFloat {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let DivideFloat {
            destination,
            left,
            right,
        } = self;

        write!(f, "R{} = {} ÷ {}", destination, left, right)
    }
}
