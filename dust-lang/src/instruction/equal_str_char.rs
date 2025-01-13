use std::fmt::{self, Display, Formatter};

use super::{Instruction, InstructionBuilder, Operand, Operation};

pub struct EqualStrChar {
    pub comparator: bool,
    pub left: Operand,
    pub right: Operand,
}

impl From<Instruction> for EqualStrChar {
    fn from(instruction: Instruction) -> Self {
        let comparator = instruction.d_field();
        let (left, right) = instruction.b_and_c_as_operands();

        EqualStrChar {
            comparator,
            left,
            right,
        }
    }
}

impl From<EqualStrChar> for Instruction {
    fn from(equal_str_char: EqualStrChar) -> Self {
        let operation = Operation::EQUAL_STR_CHAR;
        let (b_field, b_is_constant) = equal_str_char.left.as_index_and_constant_flag();
        let (c_field, c_is_constant) = equal_str_char.right.as_index_and_constant_flag();
        let d_field = equal_str_char.comparator;

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

impl Display for EqualStrChar {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let EqualStrChar {
            comparator,
            left,
            right,
        } = self;
        let operator = if *comparator { "==" } else { "≠" };

        write!(f, "if {left} {operator} {right} {{ JUMP +1 }}")
    }
}
