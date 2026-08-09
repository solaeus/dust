use std::fmt::{self, Display, Formatter};

use crate::instruction::{Address, Instruction, InstructionBuilder, OperandType, Operation};

pub struct Move {
    pub destination: u16,
    pub operand_type: OperandType,
    pub operand: Address,
    pub jump_distance: i16,
}

impl From<Instruction> for Move {
    fn from(instruction: Instruction) -> Self {
        Move {
            destination: instruction.a_field(),
            operand_type: instruction.operand_type(),
            operand: instruction.b_address(),
            jump_distance: instruction.c_field() as i16,
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
        } = r#move;

        InstructionBuilder::new(Operation::MOVE)
            .a_field(destination)
            .b_address(operand)
            .c_field(jump_distance as u16)
            .operand_type(operand_type)
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
        } = *self;

        write!(f, "reg_{destination}: {operand_type} = {operand_addres}")?;

        if jump_distance > 0 {
            write!(f, " jump {jump_distance}")?;
        }

        Ok(())
    }
}
