use std::fmt::{self, Display, Formatter};

use super::{Instruction, InstructionFields, Operand, Operation};

pub struct LessEqual {
    pub comparator: bool,
    pub left: Operand,
    pub right: Operand,
}

impl From<Instruction> for LessEqual {
    fn from(instruction: Instruction) -> Self {
        let comparator = instruction.d_field();
        let (left, right) = instruction.b_and_c_as_operands();

        LessEqual {
            comparator,
            left,
            right,
        }
    }
}

impl From<LessEqual> for Instruction {
    fn from(less_equal_byte: LessEqual) -> Self {
        let operation = Operation::LESS_EQUAL;
        let (b_field, b_is_constant) = less_equal_byte.left.as_index_and_constant_flag();
        let (c_field, c_is_constant) = less_equal_byte.right.as_index_and_constant_flag();
        let d_field = less_equal_byte.comparator;
        let b_type = less_equal_byte.left.as_type();
        let c_type = less_equal_byte.right.as_type();

        InstructionFields {
            operation,
            b_field,
            c_field,
            d_field,
            b_is_constant,
            c_is_constant,
            b_type,
            c_type,
            ..Default::default()
        }
        .build()
    }
}

impl Display for LessEqual {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let LessEqual {
            comparator,
            left,
            right,
        } = self;
        let operator = if *comparator { "≤" } else { ">" };

        write!(f, "if {left} {operator} {right} {{ JUMP +1 }}")
    }
}
