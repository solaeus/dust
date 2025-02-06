use std::fmt::{self, Display, Formatter};

use crate::{Instruction, Operation};

use super::InstructionFields;

pub struct Close {
    pub from: u16,
    pub to: u16,
}

impl From<Instruction> for Close {
    fn from(instruction: Instruction) -> Self {
        Close {
            from: instruction.b_field(),
            to: instruction.c_field(),
        }
    }
}

impl From<Close> for Instruction {
    fn from(close: Close) -> Self {
        let operation = Operation::CLOSE;
        let b_field = close.from;
        let c_field = close.to;

        InstructionFields {
            operation,
            b_field,
            c_field,
            ..Default::default()
        }
        .build()
    }
}

impl Display for Close {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let Close { from, to } = self;

        write!(f, "{from}..={to}")
    }
}
