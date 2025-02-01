use std::fmt::{self, Display, Formatter};

use crate::{Instruction, Operation};

use super::{InstructionBuilder, TypeCode};

pub struct LoadEncoded {
    pub destination: u16,
    pub value: u16,
    pub r#type: TypeCode,
    pub jump_next: bool,
}

impl From<Instruction> for LoadEncoded {
    fn from(instruction: Instruction) -> Self {
        LoadEncoded {
            destination: instruction.a_field(),
            value: instruction.b_field(),
            r#type: instruction.b_type(),
            jump_next: instruction.c_field() != 0,
        }
    }
}

impl From<LoadEncoded> for Instruction {
    fn from(load_encoded: LoadEncoded) -> Self {
        let operation = Operation::LOAD_ENCODED;
        let a_field = load_encoded.destination;
        let b_field = load_encoded.value as u16;
        let b_type = load_encoded.r#type;
        let c_field = load_encoded.jump_next as u16;

        InstructionBuilder {
            operation,
            a_field,
            b_field,
            b_type,
            c_field,
            ..Default::default()
        }
        .build()
    }
}

impl Display for LoadEncoded {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let LoadEncoded {
            destination,
            value,
            r#type,
            jump_next,
        } = self;

        match *r#type {
            TypeCode::BOOLEAN => write!(f, "R{destination} = {}", *value != 0)?,
            TypeCode::BYTE => write!(f, "R{destination} = {value}")?,
            unsupported => unsupported.panic_from_unsupported_code(),
        };

        if *jump_next {
            write!(f, " JUMP +1")?;
        }

        Ok(())
    }
}
