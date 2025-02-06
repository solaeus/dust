use std::fmt::{self, Display, Formatter};

use crate::{Instruction, Operation};

use super::{InstructionFields, TypeCode};

pub struct LoadEncoded {
    pub destination: u16,
    pub value: u16,
    pub value_type: TypeCode,
    pub jump_next: bool,
}

impl From<Instruction> for LoadEncoded {
    fn from(instruction: Instruction) -> Self {
        LoadEncoded {
            destination: instruction.a_field(),
            value: instruction.b_field(),
            value_type: instruction.b_type(),
            jump_next: instruction.c_field() != 0,
        }
    }
}

impl From<LoadEncoded> for Instruction {
    fn from(load_boolean: LoadEncoded) -> Self {
        let operation = Operation::LOAD_ENCODED;
        let a_field = load_boolean.destination;
        let b_field = load_boolean.value;
        let b_type = load_boolean.value_type;
        let c_field = load_boolean.jump_next as u16;

        InstructionFields {
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
            value_type,
            jump_next,
        } = self;

        match *value_type {
            TypeCode::BOOLEAN => {
                let boolean = *value != 0;

                write!(f, "R_BOOL_{destination} = {boolean}")?
            }
            TypeCode::BYTE => {
                let byte = *value as u8;

                write!(f, "R_BYTE_{destination} = 0x{byte:0X}")?
            }
            _ => panic!("Invalid type code {value_type} for LoadEncoded instruction"),
        }

        if *jump_next {
            write!(f, " JUMP +1")?;
        }

        Ok(())
    }
}
