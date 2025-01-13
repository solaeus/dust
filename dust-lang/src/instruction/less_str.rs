use std::fmt::{self, Display, Formatter};

use super::{Instruction, InstructionBuilder, Operand, Operation};

pub struct LessStr {
    pub comparator: bool,
    pub left: Operand,
    pub right: Operand,
}

impl From<Instruction> for LessStr {
    fn from(instruction: Instruction) -> Self {
        let comparator = instruction.d_field();
        let (left, right) = instruction.b_and_c_as_operands();

        LessStr {
            comparator,
            left,
            right,
        }
    }
}

impl From<LessStr> for Instruction {
    fn from(less_str: LessStr) -> Self {
        let operation = Operation::LESS_STR;
        let (b_field, b_is_constant) = less_str.left.as_index_and_constant_flag();
        let (c_field, c_is_constant) = less_str.right.as_index_and_constant_flag();
        let d_field = less_str.comparator;

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

impl Display for LessStr {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let LessStr {
            comparator,
            left,
            right,
        } = self;
        let operator = if *comparator { "<" } else { "≥" };

        write!(f, "if {left} {operator} {right} {{ JUMP +1 }}")
    }
}
