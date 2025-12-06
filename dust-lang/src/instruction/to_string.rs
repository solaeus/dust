use std::fmt::{self, Display, Formatter};

use crate::instruction::{Address, Instruction, InstructionFields, OperandType, Operation};

pub struct ToString {
    pub destination: u16,
    pub operand: Address,
    pub r#type: OperandType,
}

impl From<&Instruction> for ToString {
    fn from(instruction: &Instruction) -> Self {
        let destination = instruction.a_field();
        let operand = Address {
            index: instruction.b_field(),
            memory: instruction.b_memory_kind(),
        };
        let r#type = instruction.operand_type();

        ToString {
            destination,
            operand,
            r#type,
        }
    }
}

impl From<ToString> for Instruction {
    fn from(modulo: ToString) -> Self {
        let operation = Operation::TO_STRING;
        let a_field = modulo.destination;
        let Address {
            index: b_field,
            memory: b_memory_kind,
        } = modulo.operand;
        let operand_type = modulo.r#type;

        InstructionFields {
            operation,
            a_field,
            b_field,
            b_memory_kind,
            operand_type,
            ..Default::default()
        }
        .build()
    }
}

impl Display for ToString {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let ToString {
            destination,
            operand,
            r#type,
        } = self;

        write!(f, "reg_{destination} = ")?;
        operand.display(f, *r#type)?;
        write!(f, " as str")
    }
}
