use std::fmt::{self, Display, Formatter};

use tracing::error;

use crate::{Instruction, Operation};

use super::{AddressKind, Destination, InstructionFields};

pub struct LoadEncoded {
    pub destination: Destination,
    pub value: u16,
    pub r#type: AddressKind,
    pub jump_next: bool,
}

impl From<&Instruction> for LoadEncoded {
    fn from(instruction: &Instruction) -> Self {
        let destination = instruction.destination();
        let value = instruction.b_field();
        let r#type = instruction.b_kind();
        let jump_next = instruction.c_field() != 0;

        LoadEncoded {
            destination,
            value,
            r#type,
            jump_next,
        }
    }
}

impl From<LoadEncoded> for Instruction {
    fn from(load_encoded: LoadEncoded) -> Self {
        let operation = Operation::LOAD_ENCODED;
        let Destination {
            index: a_field,
            is_register: a_is_register,
        } = load_encoded.destination;
        let b_field = load_encoded.value;
        let b_kind = load_encoded.r#type;
        let c_field = load_encoded.jump_next as u16;

        InstructionFields {
            operation,
            a_field,
            a_is_register,
            b_field,
            b_kind,
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
        let destination_address = destination.as_address(r#type.r#type());

        write!(f, "{destination_address} = ")?;

        match *r#type {
            AddressKind::BOOLEAN_MEMORY => {
                let boolean = *value != 0;

                write!(f, "{boolean}")?
            }
            AddressKind::BYTE_MEMORY => {
                let byte = *value as u8;

                write!(f, "0x{byte}")?
            }
            invalid => {
                error!("Cannot display {invalid:?} here");

                write!(f, "INVALID")?
            }
        }

        if *jump_next {
            write!(f, " JUMP +1")?;
        }

        Ok(())
    }
}
