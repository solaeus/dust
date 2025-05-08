use std::fmt::{self, Display, Formatter};

use super::{Address, Destination, Instruction, InstructionFields, Operation, TypeCode};

pub struct Negate {
    pub destination: Destination,
    pub operand: Address,
}

impl From<Instruction> for Negate {
    fn from(instruction: Instruction) -> Self {
        let destination = instruction.destination();
        let operand = instruction.b_address();

        Negate {
            destination,
            operand,
        }
    }
}

impl From<Negate> for Instruction {
    fn from(negate: Negate) -> Self {
        let operation = Operation::NEGATE;
        let Destination {
            index: a_field,
            is_register: a_is_register,
        } = negate.destination;
        let Address {
            index: b_field,
            kind: b_kind,
        } = negate.operand;

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

impl Display for Negate {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let Negate {
            destination,
            operand,
        } = self;

        match operand.as_type_code() {
            TypeCode::BYTE => write!(f, "R_BYTE_{}", destination.index)?,
            TypeCode::FLOAT => write!(f, "R_FLOAT_{}", destination.index)?,
            TypeCode::INTEGER => write!(f, "R_INT_{}", destination.index)?,
            unsupported => unsupported.unsupported_write(f)?,
        }

        write!(f, " = -{operand}",)
    }
}
