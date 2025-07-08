use std::fmt::{self, Display, Formatter};

use super::{Address, Instruction, InstructionFields, OperandType, Operation};

pub struct Modulo {
    pub destination: Address,
    pub left: Address,
    pub right: Address,
    pub r#type: OperandType,
}

impl From<Instruction> for Modulo {
    fn from(instruction: Instruction) -> Self {
        let destination = instruction.destination();
        let left = instruction.b_address();
        let right = instruction.c_address();
        let r#type = instruction.operand_type();

        Modulo {
            destination,
            left,
            right,
            r#type,
        }
    }
}

impl From<Modulo> for Instruction {
    fn from(modulo: Modulo) -> Self {
        let operation = Operation::MODULO;
        let Address {
            index: a_field,
            memory: a_memory_kind,
        } = modulo.destination;
        let Address {
            index: b_field,
            memory: b_memory_kind,
        } = modulo.left;
        let Address {
            index: c_field,
            memory: c_memory_kind,
        } = modulo.right;
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
        }
        .build()
    }
}

impl Display for Modulo {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let Modulo {
            destination,
            left,
            right,
            ..
        } = self;

        write!(f, "{destination} = {left} % {right}")
    }
}
