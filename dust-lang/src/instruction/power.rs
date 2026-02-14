use std::fmt::{self, Display, Formatter};

use crate::instruction::{Address, Instruction, InstructionFields, Operation};

pub struct Power {
    pub destination: u16,
    pub base: Address,
    pub exponent: Address,
}

impl From<&Instruction> for Power {
    fn from(instruction: &Instruction) -> Self {
        let destination = instruction.a_field();
        let base = instruction.b_address();
        let exponent = instruction.c_address();

        Power {
            destination,
            base,
            exponent,
        }
    }
}

impl From<Power> for Instruction {
    fn from(power: Power) -> Self {
        let operation = Operation::POWER;
        let a_field = power.destination;
        let Address {
            index: b_field,
            memory: b_memory_kind,
        } = power.base;
        let Address {
            index: c_field,
            memory: c_memory_kind,
        } = power.exponent;

        InstructionFields {
            operation,
            a_field,
            b_field,
            b_memory_kind,
            c_field,
            c_memory_kind,
            ..Default::default()
        }
        .build()
    }
}

impl Display for Power {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let Power {
            destination,
            base,
            exponent,
        } = self;

        write!(f, "reg_{destination} = {base}^{exponent}")
    }
}
