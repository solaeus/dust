use std::fmt::{self, Display, Formatter};

use super::{Instruction, InstructionBuilder, Operand, Operation};

pub struct LessEqualStr {
    pub comparator: bool,
    pub left: Operand,
    pub right: Operand,
}

impl From<Instruction> for LessEqualStr {
    fn from(instruction: Instruction) -> Self {
        let comparator = instruction.d_field();
        let (left, right) = instruction.b_and_c_as_operands();

        LessEqualStr {
            comparator,
            left,
            right,
        }
    }
}

impl From<LessEqualStr> for Instruction {
    fn from(less_equal_str: LessEqualStr) -> Self {
        let operation = Operation::LESS_EQUAL_STR;
        let (b_field, b_is_constant) = less_equal_str.left.as_index_and_constant_flag();
        let (c_field, c_is_constant) = less_equal_str.right.as_index_and_constant_flag();
        let d_field = less_equal_str.comparator;

        InstructionBuilder {
            operation,
            b_field,
            c_field,
            d_field,
            b_is_constant,
            c_is_constant,
            ..Default::default()
        }
        .build()
    }
}

impl Display for LessEqualStr {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let LessEqualStr {
            comparator,
            left,
            right,
        } = self;
        let operator = if *comparator { "≤" } else { ">" };

        write!(f, "if {left} {operator} {right} {{ JUMP +1 }}")
    }
}
