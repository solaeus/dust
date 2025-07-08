use std::fmt::{self, Display, Formatter};

use super::{Address, Instruction, InstructionFields, OperandType, Operation};

pub struct Subtract {
    pub destination: Address,
    pub left: Address,
    pub right: Address,
    pub r#type: OperandType,
}

impl From<Instruction> for Subtract {
    fn from(instruction: Instruction) -> Self {
        let destination = instruction.destination();
        let left = instruction.b_address();
        let right = instruction.c_address();
        let r#type = instruction.operand_type();

        Subtract {
            destination,
            left,
            right,
            r#type,
        }
    }
}

impl From<Subtract> for Instruction {
    fn from(subtract: Subtract) -> Self {
        let operation = Operation::SUBTRACT;
        let Address {
            index: a_field,
            memory: a_memory_kind,
        } = subtract.destination;
        let Address {
            index: b_field,
            memory: b_memory_kind,
        } = subtract.left;
        let Address {
            index: c_field,
            memory: c_memory_kind,
        } = subtract.right;
        let operand_type = subtract.r#type;

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

impl Display for Subtract {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let Subtract {
            destination,
            left,
            right,
            ..
        } = self;

        write!(f, "{destination} = {left} - {right}")
    }
}
