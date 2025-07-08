use std::fmt::{self, Display, Formatter};

use super::{Address, Instruction, InstructionFields, OperandType, Operation};

pub struct Divide {
    pub destination: Address,
    pub left: Address,
    pub right: Address,
    pub r#type: OperandType,
}

impl From<Instruction> for Divide {
    fn from(instruction: Instruction) -> Self {
        let destination = instruction.destination();
        let left = instruction.b_address();
        let right = instruction.c_address();
        let r#type = instruction.operand_type();

        Divide {
            destination,
            left,
            right,
            r#type,
        }
    }
}

impl From<Divide> for Instruction {
    fn from(divide: Divide) -> Self {
        let operation = Operation::DIVIDE;
        let Address {
            index: a_field,
            memory: a_memory_kind,
        } = divide.destination;
        let Address {
            index: b_field,
            memory: b_memory_kind,
        } = divide.left;
        let Address {
            index: c_field,
            memory: c_memory_kind,
        } = divide.right;
        let operand_type = divide.r#type;

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

impl Display for Divide {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let Divide {
            destination,
            left,
            right,
            ..
        } = self;

        write!(f, "{destination} = {left} ÷ {right}")
    }
}
