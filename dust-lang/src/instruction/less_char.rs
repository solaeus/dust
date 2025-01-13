use std::fmt::{self, Display, Formatter};

use super::{Instruction, InstructionBuilder, Operand, Operation};

pub struct LessChar {
    pub comparator: bool,
    pub left: Operand,
    pub right: Operand,
}

impl From<Instruction> for LessChar {
    fn from(instruction: Instruction) -> Self {
        let comparator = instruction.d_field();
        let (left, right) = instruction.b_and_c_as_operands();

        LessChar {
            comparator,
            left,
            right,
        }
    }
}

impl From<LessChar> for Instruction {
    fn from(less_char: LessChar) -> Self {
        let operation = Operation::LESS_CHAR;
        let (b_field, b_is_constant) = less_char.left.as_index_and_constant_flag();
        let (c_field, c_is_constant) = less_char.right.as_index_and_constant_flag();
        let d_field = less_char.comparator;

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

impl Display for LessChar {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let LessChar {
            comparator,
            left,
            right,
        } = self;
        let operator = if *comparator { "<" } else { "≥" };

        write!(f, "if {left} {operator} {right} {{ JUMP +1 }}")
    }
}
