use std::fmt::Display;

use crate::{Instruction, Operand, Operation, TypeCode};

use super::{Destination, InstructionFields};

pub struct Not {
    pub destination: Destination,
    pub operand: Operand,
}

impl From<Instruction> for Not {
    fn from(instruction: Instruction) -> Self {
        let destination = instruction.destination();
        let operand = instruction.b_operand();

        Not {
            destination,
            operand,
        }
    }
}

impl From<Not> for Instruction {
    fn from(not: Not) -> Self {
        let operation = Operation::NOT;
        let Destination {
            index: a_field,
            is_register: a_is_register,
        } = not.destination;
        let Operand {
            index: b_field,
            kind: b_kind,
        } = not.operand;

        InstructionFields {
            operation,
            a_field,
            a_is_register,
            b_field,
            b_kind,
            ..Default::default()
        }
        .build()
    }
}

impl Display for Not {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let Not {
            destination,
            operand,
        } = self;

        match operand.as_type_code() {
            TypeCode::BOOLEAN => write!(f, "R_BOOL_{}", destination.index)?,
            unsupported => unsupported.unsupported_write(f)?,
        }

        write!(f, " = !{operand}",)
    }
}
