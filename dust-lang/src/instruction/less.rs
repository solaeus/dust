use std::fmt::{self, Display, Formatter};

use super::{Address, Instruction, InstructionFields, Operation};

pub struct Less {
    pub comparator: bool,
    pub left: Address,
    pub right: Address,
}

impl From<&Instruction> for Less {
    fn from(instruction: &Instruction) -> Self {
        let comparator = instruction.a_field() != 0;
        let left = instruction.b_address();
        let right = instruction.c_address();

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
        let Address {
            index: b_field,
            kind: b_kind,
        } = less.left;
        let Address {
            index: c_field,
            kind: c_kind,
        } = less.right;

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
