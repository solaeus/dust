use std::fmt::{self, Display, Formatter};

use crate::{Instruction, Operation};

use super::{Address, InstructionFields};

pub struct Close {
    pub from: Address,
    pub to: Address,
}

impl From<&Instruction> for Close {
    fn from(instruction: &Instruction) -> Self {
        Close {
            from: instruction.b_address(),
            to: instruction.c_address(),
        }
    }
}

impl From<Close> for Instruction {
    fn from(close: Close) -> Self {
        let operation = Operation::CLOSE;
        let Address {
            index: b_field,
            kind: b_kind,
        } = close.from;
        let Address {
            index: c_field,
            kind: c_kind,
        } = close.to;

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

impl Display for Close {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let Close { from, to } = self;

        write!(f, "{from}..={to}")
    }
}
