use std::fmt::{self, Display, Formatter};

use crate::{Instruction, Operation};

use super::{InstructionBuilder, TypeCode};

pub struct LoadInline {
    pub destination: u16,
    pub r#type: TypeCode,
    pub byte: u8,
    pub boolean: bool,
    pub jump_next: bool,
}

impl From<Instruction> for LoadInline {
    fn from(instruction: Instruction) -> Self {
        LoadInline {
            destination: instruction.a_field(),
            r#type: instruction.b_type(),
            byte: instruction.b_field() as u8,
            boolean: instruction.c_field() != 0,
            jump_next: instruction.d_field(),
        }
    }
}

impl From<LoadInline> for Instruction {
    fn from(load_boolean: LoadInline) -> Self {
        let operation = Operation::LOAD_INLINE;
        let a_field = load_boolean.destination;
        let b_type = load_boolean.r#type;
        let b_field = load_boolean.byte as u16;
        let c_field = load_boolean.boolean as u16;
        let d_field = load_boolean.jump_next;

        InstructionBuilder {
            operation,
            a_field,
            b_field,
            b_type,
            c_field,
            d_field,
            ..Default::default()
        }
        .build()
    }
}

impl Display for LoadInline {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let LoadInline {
            destination,
            r#type,
            byte,
            boolean,
            jump_next,
        } = self;

        match *r#type {
            TypeCode::BYTE => write!(f, "R_BYTE_{destination} = {byte}")?,
            TypeCode::BOOLEAN => write!(f, "R_BOOL_{destination} = {boolean}")?,
            _ => write!(f, "INVALID")?,
        }

        if *jump_next {
            write!(f, " JUMP +1")?;
        }

        Ok(())
    }
}
