use std::fmt::{self, Display, Formatter};

use super::{Address, Instruction, InstructionFields, Operation};

pub struct LessEqual {
    pub comparator: bool,
    pub left: Address,
    pub right: Address,
}

impl From<Instruction> for LessEqual {
    fn from(instruction: Instruction) -> Self {
        let comparator = instruction.a_field() != 0;
        let left = instruction.b_address();
        let right = instruction.c_address();

        LessEqual {
            comparator,
            left,
            right,
        }
    }
}

impl From<LessEqual> for Instruction {
    fn from(less_equal: LessEqual) -> Self {
        let operation = Operation::LESS_EQUAL;
        let Address {
            index: b_field,
            kind: b_kind,
        } = less_equal.left;
        let Address {
            index: c_field,
            kind: c_kind,
        } = less_equal.right;

        InstructionFields {
            operation,
            b_field,
            b_kind,
            c_field,
            c_kind,
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
            right,
        } = self;
        let operator = if *comparator { "≤" } else { ">" };

        write!(f, "if {left} {operator} {right} {{ JUMP +1 }}")
    }
}
