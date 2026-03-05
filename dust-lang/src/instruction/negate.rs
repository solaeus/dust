use std::fmt::{self, Display, Formatter};

use crate::instruction::{Instruction, InstructionBuilder, MemoryKind, OperandType, Operation};

pub struct Negate {
    pub destination: u16,
    pub operand_type: OperandType,
    pub operand_memory: MemoryKind,
    pub operand_index: u16,
}

impl From<&Instruction> for Negate {
    fn from(instruction: &Instruction) -> Self {
        Negate {
            destination: instruction.a_field(),
            operand_type: instruction.operand_type(),
            operand_memory: instruction.b_memory(),
            operand_index: instruction.b_field(),
        }
    }
}

impl From<Negate> for Instruction {
    fn from(negate: Negate) -> Self {
        let Negate {
            destination,
            operand_type,
            operand_memory,
            operand_index,
        } = negate;

        InstructionBuilder::new(Operation::NEGATE)
            .a_field(destination)
            .b_memory(operand_memory)
            .b_field(operand_index)
            .operand_type(operand_type)
            .build()
    }
}

impl Display for Negate {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let Negate {
            destination,
            operand_type,
            operand_memory,
            operand_index,
        } = *self;
        let operator = match operand_type {
            OperandType::BOOLEAN => "!",
            _ => "-",
        };
        let operand_memory = operand_memory.as_string(operand_type);

        write!(
            f,
            "reg_{destination} = {operator}{operand_memory}_{operand_index}"
        )
    }
}
