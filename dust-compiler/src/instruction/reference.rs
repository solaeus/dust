use std::fmt::{self, Display, Formatter};

use crate::instruction::{
    Address, Instruction, InstructionBuilder, MemoryKind, OperandType, Operation,
};

pub struct Reference {
    pub destination: u16,
    pub operand_type: OperandType,
    pub source: Address,
    pub jump_distance: u16,
    pub jump_forward: bool,
}

impl From<Instruction> for Reference {
    fn from(instruction: Instruction) -> Self {
        Reference {
            destination: instruction.a_field(),
            operand_type: instruction.operand_type(),
            source: instruction.b_address(),
            jump_distance: instruction.c_field(),
            jump_forward: instruction.c_memory().0 != 0,
        }
    }
}

impl From<Reference> for Instruction {
    fn from(reference: Reference) -> Self {
        let Reference {
            destination,
            operand_type,
            source,
            jump_distance,
            jump_forward,
        } = reference;

        InstructionBuilder::new(Operation::REFERENCE)
            .a_field(destination)
            .b_address(source)
            .c_field(jump_distance)
            .c_memory(MemoryKind(jump_forward as u8))
            .operand_type(operand_type)
            .build()
    }
}

impl Display for Reference {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let Reference {
            destination,
            operand_type,
            source,
            jump_distance,
            jump_forward,
        } = *self;

        write!(f, "reg_{destination}: &{operand_type} = &{source};")?;

        if jump_distance > 0 {
            let direction = if jump_forward { "+" } else { "-" };

            write!(f, " jump {direction}{jump_distance}")?;
        }

        Ok(())
    }
}
