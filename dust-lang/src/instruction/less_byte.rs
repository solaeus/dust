use std::fmt::{self, Display, Formatter};

use super::{Instruction, InstructionBuilder, Operand, Operation};

pub struct LessByte {
    pub comparator: bool,
    pub left: Operand,
    pub right: Operand,
}

impl From<Instruction> for LessByte {
    fn from(instruction: Instruction) -> Self {
        let comparator = instruction.d_field();
        let (left, right) = instruction.b_and_c_as_operands();

        LessByte {
            comparator,
            left,
            right,
        }
    }
}

impl From<LessByte> for Instruction {
    fn from(less_byte: LessByte) -> Self {
        let operation = Operation::LESS_BYTE;
        let (b_field, b_is_constant) = less_byte.left.as_index_and_constant_flag();
        let (c_field, c_is_constant) = less_byte.right.as_index_and_constant_flag();
        let d_field = less_byte.comparator;

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

impl Display for LessByte {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let LessByte {
            comparator,
            left,
            right,
        } = self;
        let operator = if *comparator { "<" } else { "≥" };

        write!(f, "if {left} {operator} {right} {{ JUMP +1 }}")
    }
}
