use std::fmt::{self, Display, Formatter};

use super::{Address, Instruction, InstructionFields, OperandType, Operation};

pub struct Move {
    pub destination: u16,
    pub operand: Address,
    pub r#type: OperandType,
    pub jump_next: bool,
}

impl From<Instruction> for Move {
    fn from(instruction: Instruction) -> Self {
        let destination = instruction.a_field();
        let operand = instruction.b_address();
        let r#type = instruction.operand_type();
        let jump_next = instruction.c_field() != 0;

        Move {
            destination,
            operand,
            r#type,
            jump_next,
        }
    }
}

impl From<Move> for Instruction {
    fn from(load: Move) -> Self {
        let a_field = load.destination;
        let Address {
            index: b_field,
            memory: b_memory_kind,
        } = load.operand;
        let c_field = load.jump_next as u16;
        let operand_type = load.r#type;

        InstructionFields {
            operation: Operation::MOVE,
            a_field,
            b_field,
            b_memory_kind,
            c_field,
            operand_type,
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
            jump_next,
            r#type,
        } = self;

        write!(f, "reg_{destination} = ")?;
        operand.display(f, *r#type)?;

        if *jump_next {
            write!(f, " jump +1")?;
        }

        Ok(())
    }
}
