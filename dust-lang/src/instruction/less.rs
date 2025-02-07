use std::fmt::{self, Display, Formatter};

use super::{Instruction, InstructionFields, Operand, Operation};

pub struct Less {
    pub comparator: bool,
    pub left: Operand,
    pub right: Operand,
}

impl From<Instruction> for Less {
    fn from(instruction: Instruction) -> Self {
        let comparator = instruction.d_field();
        let (left, right) = instruction.b_and_c_as_operands();

        Less {
            comparator,
            left,
            right,
        }
    }
}

impl From<Less> for Instruction {
    fn from(less: Less) -> Self {
        let operation = Operation::LESS;
        let (b_field, b_is_constant) = less.left.as_index_and_constant_flag();
        let (c_field, c_is_constant) = less.right.as_index_and_constant_flag();
        let d_field = less.comparator;
        let b_type = less.left.as_type();
        let c_type = less.right.as_type();

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

impl Display for Less {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let Less {
            comparator,
            left,
            right,
        } = self;
        let operator = if *comparator { "<" } else { "≥" };

        write!(f, "if {left} {operator} {right} {{ JUMP +1 }}")
    }
}
