use std::fmt::{self, Display, Formatter};

use super::{Instruction, InstructionBuilder, Operand, Operation};

pub struct EqualStr {
    pub comparator: bool,
    pub left: Operand,
    pub right: Operand,
}

impl From<Instruction> for EqualStr {
    fn from(instruction: Instruction) -> Self {
        let comparator = instruction.d_field();
        let (left, right) = instruction.b_and_c_as_operands();

        EqualStr {
            comparator,
            left,
            right,
        }
    }
}

impl From<EqualStr> for Instruction {
    fn from(equal_str: EqualStr) -> Self {
        let operation = Operation::EQUAL_STR;
        let (b_field, b_is_constant) = equal_str.left.as_index_and_constant_flag();
        let (c_field, c_is_constant) = equal_str.right.as_index_and_constant_flag();
        let d_field = equal_str.comparator;

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

impl Display for EqualStr {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let EqualStr {
            comparator,
            left,
            right,
        } = self;
        let operator = if *comparator { "==" } else { "≠" };

        write!(f, "if {left} {operator} {right} {{ JUMP +1 }}")
    }
}
