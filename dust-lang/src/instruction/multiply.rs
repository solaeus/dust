use std::fmt::{self, Display, Formatter};

use super::{Address, Instruction, InstructionFields, OperandType, Operation};

pub struct Multiply {
    pub destination: Address,
    pub left: Address,
    pub right: Address,
    pub r#type: OperandType,
}

impl From<Instruction> for Multiply {
    fn from(instruction: Instruction) -> Self {
        let destination = instruction.destination();
        let left = instruction.b_address();
        let right = instruction.c_address();
        let r#type = instruction.operand_type();

        Multiply {
            destination,
            left,
            right,
            r#type,
        }
    }
}

impl From<Multiply> for Instruction {
    fn from(multiply: Multiply) -> Self {
        let operation = Operation::MULTIPLY;
        let Address {
            index: a_field,
            memory: a_memory_kind,
        } = multiply.destination;
        let Address {
            index: b_field,
            memory: b_memory_kind,
        } = multiply.left;
        let Address {
            index: c_field,
            memory: c_memory_kind,
        } = multiply.right;
        let operand_type = multiply.r#type;

        InstructionFields {
            operation,
            a_field,
            a_memory_kind,
            b_field,
            b_memory_kind,
            c_field,
            c_memory_kind,
            operand_type,
        }
        .build()
    }
}

impl Display for Multiply {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let Multiply {
            destination,
            left,
            right,
            r#type,
        } = self;

        destination.display(f, *r#type)?;
        write!(f, " = ")?;
        left.display(f, *r#type)?;
        write!(f, " * ")?;
        right.display(f, *r#type)?;
        Ok(())
    }
}
