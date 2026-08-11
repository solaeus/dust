use std::fmt::{self, Display, Formatter};

use crate::instruction::{
    Address, Instruction, InstructionBuilder, MemoryKind, OperandType, Operation,
};

pub struct Move {
    pub destination: u16,
    pub operand_type: OperandType,
    pub operand: Address,
    pub jump_distance: u16,
    pub jump_forward: bool,
}

impl From<Instruction> for Move {
    fn from(instruction: Instruction) -> Self {
        Move {
            destination: instruction.a_field() as u16,
            operand_type: instruction.operand_type(),
            operand: instruction.b_address(),
            jump_distance: instruction.c_field() as u16,
            jump_forward: instruction.c_memory().0 != 0,
        }
    }
}

impl From<Move> for Instruction {
    fn from(r#move: Move) -> Self {
        let Move {
            destination,
            operand_type,
            operand,
            jump_distance,
            jump_forward,
        } = r#move;

        InstructionBuilder::new(Operation::MOVE)
            .operand_type(operand_type)
            .a_field(destination)
            .b_address(operand)
            .c_field(jump_distance)
            .c_memory(MemoryKind(jump_forward as u8))
            .build()
    }
}

impl Display for Move {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let Move {
            destination,
            operand_type,
            operand: operand_addres,
            jump_distance,
            jump_forward,
        } = *self;

        write!(f, "reg_{destination}: {operand_type} = {operand_addres}")?;

        let operator = if jump_forward { "+" } else { "-" };

        if jump_distance > 0 {
            write!(f, " jump {operator}{jump_distance}")?;
        }

        Ok(())
    }
}
