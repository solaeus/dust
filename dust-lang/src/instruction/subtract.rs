use std::fmt::{self, Display, Formatter};

use super::{Address, Destination, Instruction, InstructionFields, Operation, TypeCode};

pub struct Subtract {
    pub destination: Destination,
    pub left: Address,
    pub right: Address,
}

impl From<Instruction> for Subtract {
    fn from(instruction: Instruction) -> Self {
        let destination = instruction.destination();
        let left = instruction.b_address();
        let right = instruction.c_address();

        Subtract {
            destination,
            left,
            right,
        }
    }
}

impl From<Subtract> for Instruction {
    fn from(subtract: Subtract) -> Self {
        let operation = Operation::SUBTRACT;
        let Destination {
            index: a_field,
            is_register: a_is_register,
        } = subtract.destination;
        let Address {
            index: b_field,
            kind: b_kind,
        } = subtract.left;
        let Address {
            index: c_field,
            kind: c_kind,
        } = subtract.right;

        InstructionFields {
            operation,
            a_field,
            a_is_register,
            b_field,
            b_kind,
            c_field,
            c_kind,
        }
        .build()
    }
}

impl Display for Subtract {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let Subtract {
            destination,
            left,
            right,
        } = self;

        match left.as_type_code() {
            TypeCode::BYTE => write!(f, "R_BYTE_{}", destination.index)?,
            TypeCode::FLOAT => write!(f, "R_FLOAT_{}", destination.index)?,
            TypeCode::INTEGER => write!(f, "R_INT_{}", destination.index)?,
            unsupported => unsupported.unsupported_write(f)?,
        }

        write!(f, " = {left} - {right}",)
    }
}
