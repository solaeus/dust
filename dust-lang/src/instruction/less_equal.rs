use std::fmt::{self, Display, Formatter};

use super::{Instruction, InstructionBuilder, Operand, Operation, TypeCode};

pub struct LessEqual {
    pub comparator: bool,
    pub left: Operand,
    pub left_type: TypeCode,
    pub right: Operand,
    pub right_type: TypeCode,
}

impl From<Instruction> for LessEqual {
    fn from(instruction: Instruction) -> Self {
        let comparator = instruction.d_field();
        let (left, right) = instruction.b_and_c_as_operands();
        let left_type = instruction.b_type();
        let right_type = instruction.c_type();

        LessEqual {
            comparator,
            left,
            left_type,
            right,
            right_type,
        }
    }
}

impl From<LessEqual> for Instruction {
    fn from(less_equal_byte: LessEqual) -> Self {
        let operation = Operation::LESS_EQUAL;
        let (b_field, b_is_constant) = less_equal_byte.left.as_index_and_constant_flag();
        let (c_field, c_is_constant) = less_equal_byte.right.as_index_and_constant_flag();
        let d_field = less_equal_byte.comparator;
        let b_type = less_equal_byte.left_type;
        let c_type = less_equal_byte.right_type;

        InstructionBuilder {
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
            left_type: _,
            right,
            right_type: _,
        } = self;
        let operator = if *comparator { "≤" } else { ">" };

        write!(f, "if {left} {operator} {right} {{ JUMP +1 }}")
    }
}
