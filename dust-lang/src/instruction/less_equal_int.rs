use std::fmt::{self, Display, Formatter};

use super::{Instruction, InstructionBuilder, Operand, Operation};

pub struct LessEqualInt {
    pub comparator: bool,
    pub left: Operand,
    pub right: Operand,
}

impl From<Instruction> for LessEqualInt {
    fn from(instruction: Instruction) -> Self {
        let comparator = instruction.d_field();
        let (left, right) = instruction.b_and_c_as_operands();

        LessEqualInt {
            comparator,
            left,
            right,
        }
    }
}

impl From<LessEqualInt> for Instruction {
    fn from(less_equal_int: LessEqualInt) -> Self {
        let operation = Operation::LESS_EQUAL_INT;
        let (b_field, b_is_constant) = less_equal_int.left.as_index_and_constant_flag();
        let (c_field, c_is_constant) = less_equal_int.right.as_index_and_constant_flag();
        let d_field = less_equal_int.comparator;

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

impl Display for LessEqualInt {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let LessEqualInt {
            comparator,
            left,
            right,
        } = self;
        let operator = if *comparator { "≤" } else { ">" };

        write!(f, "if {left} {operator} {right} {{ JUMP +1 }}")
    }
}
