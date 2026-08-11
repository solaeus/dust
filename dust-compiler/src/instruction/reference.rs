use std::fmt::{self, Display, Formatter};

use crate::instruction::{Instruction, InstructionBuilder, Operation};

pub struct Reference {
    pub destination: u16,
    pub start_register: u16,
    pub end_register: u16,
}

impl From<Instruction> for Reference {
    fn from(instruction: Instruction) -> Self {
        Reference {
            destination: instruction.a_field() as u16,
            start_register: instruction.b_field() as u16,
            end_register: instruction.c_field() as u16,
        }
    }
}

impl From<Reference> for Instruction {
    fn from(reference: Reference) -> Self {
        let Reference {
            destination,
            start_register,
            end_register,
        } = reference;

        InstructionBuilder::new(Operation::REFERENCE)
            .a_field(destination)
            .b_field(start_register)
            .c_field(end_register)
            .build()
    }
}

impl Display for Reference {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let Reference {
            destination,
            start_register,
            end_register,
        } = *self;

        write!(
            f,
            "reg_{destination}: &T = &reg_{start_register}..={end_register};"
        )?;

        Ok(())
    }
}
