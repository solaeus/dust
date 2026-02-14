use std::fmt::{self, Display, Formatter};

use crate::instruction::{Instruction, InstructionFields, Operation};

pub struct Reference {
    pub destination: u16,
    pub start: u16,
    pub length: u16,
}

impl From<&Instruction> for Reference {
    fn from(instruction: &Instruction) -> Self {
        let destination = instruction.a_field();
        let start = instruction.b_field();
        let length = instruction.c_field();

        Reference {
            destination,
            start,
            length,
        }
    }
}

impl From<Reference> for Instruction {
    fn from(reference: Reference) -> Self {
        let a_field = reference.destination;
        let b_field = reference.start;
        let c_field = reference.length;

        InstructionFields {
            operation: Operation::REFERENCE,
            a_field,
            b_field,
            c_field,
            ..Default::default()
        }
        .build()
    }
}

impl Display for Reference {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let Reference {
            destination,
            start,
            length,
        } = self;
        let end = start + length;

        write!(f, "reg_{destination} = reg_{start}..reg_{end}")
    }
}
