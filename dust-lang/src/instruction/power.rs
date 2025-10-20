use std::fmt::{self, Display, Formatter};

use crate::instruction::{Address, Instruction, InstructionFields, OperandType, Operation};

pub struct Power {
    pub destination: Address,
    pub base: Address,
    pub exponent: Address,
    pub r#type: OperandType,
}

impl From<Instruction> for Power {
    fn from(instruction: Instruction) -> Self {
        let destination = instruction.destination();
        let base = instruction.b_address();
        let exponent = instruction.c_address();
        let r#type = instruction.operand_type();

        Power {
            destination,
            base,
            exponent,
            r#type,
        }
    }
}

impl From<Power> for Instruction {
    fn from(modulo: Power) -> Self {
        let operation = Operation::POWER;
        let Address {
            index: a_field,
            memory: a_memory_kind,
        } = modulo.destination;
        let Address {
            index: b_field,
            memory: b_memory_kind,
        } = modulo.base;
        let Address {
            index: c_field,
            memory: c_memory_kind,
        } = modulo.exponent;
        let operand_type = modulo.r#type;

        InstructionFields {
            operation,
            a_field,
            a_memory_kind,
            b_field,
            b_memory_kind,
            c_field,
            c_memory_kind,
            operand_type,
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
            r#type,
        } = self;

        destination.display(f, *r#type)?;
        write!(f, " = ")?;
        base.display(f, *r#type)?;
        write!(f, "^")?;
        exponent.display(f, *r#type)
    }
}
