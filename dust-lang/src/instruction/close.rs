use std::fmt::{self, Display, Formatter};

use crate::{Instruction, Operation};

use super::{InstructionFields, Operand};

pub struct Close {
    pub from: Operand,
    pub to: Operand,
}

impl From<&Instruction> for Close {
    fn from(instruction: &Instruction) -> Self {
        Close {
            from: instruction.b_operand(),
            to: instruction.c_operand(),
        }
    }
}

impl From<Close> for Instruction {
    fn from(close: Close) -> Self {
        let operation = Operation::CLOSE;
        let Operand {
            index: b_field,
            kind: b_kind,
        } = close.from;
        let Operand {
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
