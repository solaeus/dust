use std::fmt::Display;

use crate::{Address, Instruction, Operation, r#type::TypeKind};

use super::{Destination, InstructionFields};

pub struct Not {
    pub destination: Destination,
    pub operand: Address,
}

impl From<Instruction> for Not {
    fn from(instruction: Instruction) -> Self {
        let destination = instruction.destination();
        let operand = instruction.b_address();

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
        let Address {
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

        destination.display(f, TypeKind::Boolean)?;
        write!(f, " = !{operand}",)
    }
}
