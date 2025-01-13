use std::fmt::{self, Display, Formatter};

use super::{Instruction, InstructionBuilder, Operand, Operation};

pub struct EqualBool {
    pub comparator: bool,
    pub left: Operand,
    pub right: Operand,
}

impl From<Instruction> for EqualBool {
    fn from(instruction: Instruction) -> Self {
        let comparator = instruction.d_field();
        let (left, right) = instruction.b_and_c_as_operands();

        EqualBool {
            comparator,
            left,
            right,
        }
    }
}

impl From<EqualBool> for Instruction {
    fn from(equal_bool: EqualBool) -> Self {
        let operation = Operation::EQUAL_BOOL;
        let (b_field, b_is_constant) = equal_bool.left.as_index_and_constant_flag();
        let (c_field, c_is_constant) = equal_bool.right.as_index_and_constant_flag();
        let d_field = equal_bool.comparator;

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

impl Display for EqualBool {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let EqualBool {
            comparator,
            left,
            right,
        } = self;
        let operator = if *comparator { "==" } else { "≠" };

        write!(f, "if {left} {operator} {right} {{ JUMP +1 }}")
    }
}
