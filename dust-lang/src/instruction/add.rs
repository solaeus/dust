use std::fmt::{self, Display, Formatter};

use super::{Address, Instruction, InstructionFields, OperandType, Operation};

pub struct Add {
    pub destination: Address,
    pub left: Address,
    pub right: Address,
    pub r#type: OperandType,
}

impl From<&Instruction> for Add {
    fn from(instruction: &Instruction) -> Self {
        let destination = instruction.destination();
        let left = instruction.b_address();
        let right = instruction.c_address();
        let r#type = instruction.operand_type();

        Add {
            destination,
            left,
            right,
            r#type,
        }
    }
}

impl From<Add> for Instruction {
    fn from(add: Add) -> Self {
        let operation = Operation::ADD;
        let Address {
            index: a_field,
            memory: a_memory_kind,
        } = add.destination;
        let Address {
            index: b_field,
            memory: b_memory_kind,
        } = add.left;
        let Address {
            index: c_field,
            memory: c_memory_kind,
        } = add.right;
        let operand_type = add.r#type;

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

impl Display for Add {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let Add {
            destination,
            left,
            right,
            r#type,
        } = self;

        let destination_type = r#type.destination_type();
        let left_type = r#type.b_type();
        let right_type = r#type.c_type();

        destination.display(f, destination_type)?;
        write!(f, " = ")?;
        left.display(f, left_type)?;
        write!(f, " + ")?;
        right.display(f, right_type)
    }
}
