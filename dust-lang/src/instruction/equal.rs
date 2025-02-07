use std::fmt::{self, Display, Formatter};

use super::{Instruction, InstructionFields, Operand, Operation};

pub struct Equal {
    pub comparator: bool,
    pub left: Operand,
    pub right: Operand,
}

impl From<Instruction> for Equal {
    fn from(instruction: Instruction) -> Self {
        let comparator = instruction.d_field();
        let (left, right) = instruction.b_and_c_as_operands();

        Equal {
            comparator,
            left,
            right,
        }
    }
}

impl From<Equal> for Instruction {
    fn from(equal_bool: Equal) -> Self {
        let operation = Operation::EQUAL;
        let (b_field, b_is_constant) = equal_bool.left.as_index_and_constant_flag();
        let (c_field, c_is_constant) = equal_bool.right.as_index_and_constant_flag();
        let d_field = equal_bool.comparator;
        let b_type = equal_bool.left.as_type();
        let c_type = equal_bool.right.as_type();

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

impl Display for Equal {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let Equal {
            comparator,
            left,
            right,
        } = self;
        let operator = if *comparator { "==" } else { "≠" };

        write!(f, "if {left} {operator} {right} {{ JUMP +1 }}")
    }
}
