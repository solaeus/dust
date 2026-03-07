use std::fmt::{self, Display, Formatter};

use crate::instruction::{Instruction, InstructionBuilder, MemoryKind, OperandType, Operation};

pub struct Move {
    pub destination: u16,
    pub operand_type: OperandType,
    pub operand_memory: MemoryKind,
    pub operand_index: u16,
    pub jump_distance: u16,
    pub jump_forward: bool,
}

impl From<&Instruction> for Move {
    fn from(instruction: &Instruction) -> Self {
        Move {
            destination: instruction.a_field(),
            operand_type: instruction.operand_type(),
            operand_memory: instruction.b_memory(),
            operand_index: instruction.b_field(),
            jump_distance: instruction.c_field(),
            jump_forward: instruction.c_memory().0 != 0,
        }
    }
}

impl From<Move> for Instruction {
    fn from(r#move: Move) -> Self {
        let Move {
            destination,
            operand_type,
            operand_memory,
            operand_index,
            jump_distance,
            jump_forward,
        } = r#move;

        InstructionBuilder::new(Operation::MOVE)
            .a_field(destination)
            .b_memory(operand_memory)
            .b_field(operand_index)
            .c_field(jump_distance)
            .c_memory(MemoryKind(jump_forward as u8))
            .operand_type(operand_type)
            .build()
    }
}

impl Display for Move {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let Move {
            destination,
            operand_type,
            operand_memory,
            operand_index,
            jump_distance,
            jump_forward,
        } = *self;

        write!(
            f,
            "reg_{destination}: {operand_type} = {operand_memory}_{operand_index}"
        )?;

        if jump_distance > 0 {
            let direction = if jump_forward { "+" } else { "-" };

            write!(f, " jump {direction}{jump_distance}")?;
        }

        Ok(())
    }
}
