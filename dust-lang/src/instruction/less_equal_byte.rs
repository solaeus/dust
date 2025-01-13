use std::fmt::{self, Display, Formatter};

use super::{Instruction, InstructionBuilder, Operand, Operation};

pub struct LessEqualByte {
    pub comparator: bool,
    pub left: Operand,
    pub right: Operand,
}

impl From<Instruction> for LessEqualByte {
    fn from(instruction: Instruction) -> Self {
        let comparator = instruction.d_field();
        let (left, right) = instruction.b_and_c_as_operands();

        LessEqualByte {
            comparator,
            left,
            right,
        }
    }
}

impl From<LessEqualByte> for Instruction {
    fn from(less_equal_byte: LessEqualByte) -> Self {
        let operation = Operation::LESS_EQUAL_BYTE;
        let (b_field, b_is_constant) = less_equal_byte.left.as_index_and_constant_flag();
        let (c_field, c_is_constant) = less_equal_byte.right.as_index_and_constant_flag();
        let d_field = less_equal_byte.comparator;

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

impl Display for LessEqualByte {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let LessEqualByte {
            comparator,
            left,
            right,
        } = self;
        let operator = if *comparator { "≤" } else { ">" };

        write!(f, "if {left} {operator} {right} {{ JUMP +1 }}")
    }
}
