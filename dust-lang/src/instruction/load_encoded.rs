use std::fmt::{self, Display, Formatter};

use crate::{Instruction, Operation, r#type::TypeKind};

use super::{Address, Destination, InstructionFields};

pub struct LoadEncoded {
    pub destination: Destination,
    pub value: Address,
    pub jump_next: bool,
}

impl From<Instruction> for LoadEncoded {
    fn from(instruction: Instruction) -> Self {
        let destination = instruction.destination();
        let value = instruction.b_address();
        let jump_next = instruction.c_field() != 0;

        LoadEncoded {
            destination,
            value,
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
        let Address {
            index: b_field,
            kind: b_kind,
        } = load_encoded.value;
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
            jump_next,
        } = self;
        let r#type = value.r#type();
        let destination_address = destination.as_address(r#type);

        write!(f, "{destination_address} = ")?;

        match r#type {
            TypeKind::Boolean => {
                let boolean = value.index != 0;

                write!(f, "{boolean}")?
            }
            TypeKind::Byte => {
                let byte = value.index as u8;

                write!(f, "0x{byte}")?
            }
            invalid => invalid.write_invalid(f)?,
        }

        if *jump_next {
            write!(f, " JUMP +1")?;
        }

        Ok(())
    }
}
