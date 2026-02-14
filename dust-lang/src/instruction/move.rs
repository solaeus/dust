use std::fmt::{self, Display, Formatter};

use crate::instruction::MemoryKind;

use super::{Address, Instruction, InstructionFields, Operation};

pub struct Move {
    pub destination: u16,
    pub operand: Address,
    pub jump_distance: u16,
    pub jump_is_positive: bool,
}

impl From<&Instruction> for Move {
    fn from(instruction: &Instruction) -> Self {
        let destination = instruction.a_field();
        let operand = instruction.b_address();
        let jump_distance = instruction.c_field();
        let jump_is_positive = instruction.c_memory_kind().0 != 0;

        Move {
            destination,
            operand,
            jump_distance,
            jump_is_positive,
        }
    }
}

impl From<Move> for Instruction {
    fn from(r#move: Move) -> Self {
        let a_field = r#move.destination;
        let Address {
            index: b_field,
            memory: b_memory_kind,
        } = r#move.operand;
        let c_field = r#move.jump_distance;
        let c_memory_kind = MemoryKind(r#move.jump_is_positive as u8);

        InstructionFields {
            operation: Operation::MOVE,
            a_field,
            b_field,
            b_memory_kind,
            c_field,
            c_memory_kind,
            ..Default::default()
        }
        .build()
    }
}

impl Display for Move {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let Move {
            destination,
            operand,
            jump_distance,
            jump_is_positive,
        } = self;

        write!(f, "reg_{destination} = {operand}")?;

        if *jump_distance > 0 {
            if *jump_is_positive {
                write!(f, " jump +{jump_distance}")?;
            } else {
                write!(f, " jump -{jump_distance}")?;
            }
        }

        Ok(())
    }
}
