use std::fmt::{self, Display, Formatter};

use crate::instruction::{Instruction, InstructionBuilder, MemoryKind, OperandType, Operation};

pub struct Power {
    pub destination: u16,
    pub operand_type: OperandType,
    pub base_memory: MemoryKind,
    pub base_index: u16,
    pub exponent_memory: MemoryKind,
    pub exponent_index: u16,
}

impl From<&Instruction> for Power {
    fn from(instruction: &Instruction) -> Self {
        Power {
            destination: instruction.a_field(),
            operand_type: instruction.operand_type(),
            base_memory: instruction.b_memory(),
            base_index: instruction.b_field(),
            exponent_memory: instruction.c_memory(),
            exponent_index: instruction.c_field(),
        }
    }
}

impl From<Power> for Instruction {
    fn from(power: Power) -> Self {
        let Power {
            destination,
            operand_type,
            base_memory,
            base_index,
            exponent_memory,
            exponent_index,
        } = power;

        InstructionBuilder::new(Operation::POWER)
            .a_field(destination)
            .operand_type(operand_type)
            .b_memory(base_memory)
            .b_field(base_index)
            .c_memory(exponent_memory)
            .c_field(exponent_index)
            .build()
    }
}

impl Display for Power {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let Power {
            destination,
            operand_type,
            base_memory,
            base_index,
            exponent_memory,
            exponent_index,
        } = *self;

        write!(
            f,
            "reg_{destination}: {operand_type} = {base_memory}_{base_index} ^ {exponent_memory}_{exponent_index}"
        )
    }
}
