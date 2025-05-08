use std::fmt::{self, Display, Formatter};

use super::{Address, Instruction, InstructionFields, Operation};

pub struct Equal {
    pub comparator: bool,
    pub left: Address,
    pub right: Address,
}

impl From<Instruction> for Equal {
    fn from(instruction: Instruction) -> Self {
        let comparator = instruction.a_field() != 0;
        let left = instruction.b_address();
        let right = instruction.c_address();

        Equal {
            comparator,
            left,
            right,
        }
    }
}

impl From<Equal> for Instruction {
    fn from(equal: Equal) -> Self {
        let operation = Operation::EQUAL;
        let a_field = equal.comparator as u16;
        let Address {
            index: b_field,
            kind: b_kind,
        } = equal.left;
        let Address {
            index: c_field,
            kind: c_kind,
        } = equal.right;

        InstructionFields {
            operation,
            a_field,
            b_field,
            b_kind,
            c_field,
            c_kind,
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
