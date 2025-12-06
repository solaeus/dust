use std::fmt::{self, Display, Formatter};

use crate::instruction::{Address, Instruction, InstructionFields, OperandType, Operation};

pub struct Power {
    pub destination: u16,
    pub base: Address,
    pub exponent: Address,
    pub r#type: OperandType,
}

impl From<&Instruction> for Power {
    fn from(instruction: &Instruction) -> Self {
        let destination = instruction.a_field();
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
        let a_field = modulo.destination;
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

        write!(f, "reg_{destination} = ")?;
        base.display(f, *r#type)?;
        write!(f, "^")?;
        exponent.display(f, *r#type)
    }
}
