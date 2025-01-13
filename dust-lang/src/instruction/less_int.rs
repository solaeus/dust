use std::fmt::{self, Display, Formatter};

use super::{Instruction, InstructionBuilder, Operand, Operation};

pub struct LessInt {
    pub comparator: bool,
    pub left: Operand,
    pub right: Operand,
}

impl From<Instruction> for LessInt {
    fn from(instruction: Instruction) -> Self {
        let comparator = instruction.d_field();
        let (left, right) = instruction.b_and_c_as_operands();

        LessInt {
            comparator,
            left,
            right,
        }
    }
}

impl From<LessInt> for Instruction {
    fn from(less_int: LessInt) -> Self {
        let operation = Operation::LESS_INT;
        let (b_field, b_is_constant) = less_int.left.as_index_and_constant_flag();
        let (c_field, c_is_constant) = less_int.right.as_index_and_constant_flag();
        let d_field = less_int.comparator;

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

impl Display for LessInt {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let LessInt {
            comparator,
            left,
            right,
        } = self;
        let operator = if *comparator { "<" } else { "≥" };

        write!(f, "if {left} {operator} {right} {{ JUMP +1 }}")
    }
}
