use std::fmt::{self, Display, Formatter};

use super::{Address, Instruction, InstructionFields, Operation, SmallType};

pub struct Move {
    pub destination: u16,
    pub operand_type: SmallType,
    pub operand: Address,
    pub secondary_index: u16,
}

impl From<&Instruction> for Move {
    fn from(instruction: &Instruction) -> Self {
        let destination = instruction.a_field();
        let operand_type = instruction.operand_type();
        let operand = instruction.b_address();
        let secondary_operand = instruction.c_field();

        Move {
            destination,
            operand_type,
            operand,
            secondary_index: secondary_operand,
        }
    }
}

impl From<Move> for Instruction {
    fn from(r#move: Move) -> Self {
        let a_field = r#move.destination;
        let operand_type = r#move.operand_type;
        let Address {
            index: b_field,
            memory: b_memory_kind,
        } = r#move.operand;
        let c_field = r#move.secondary_index;

        InstructionFields {
            operation: Operation::MOVE,
            operand_type,
            a_field,
            b_field,
            b_memory_kind,
            c_field,
            ..Default::default()
        }
        .build()
    }
}

impl Display for Move {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let Move {
            destination,
            operand_type,
            operand,
            secondary_index,
        } = *self;

        write!(f, "reg_{destination}: {operand_type} = {operand}")?;

        let memory_kind = operand.memory;

        match operand_type {
            SmallType::U_128 | SmallType::I_128 => write!(f, " & {memory_kind}_{secondary_index}")?,
            SmallType::STRUCT => write!(f, "..={memory_kind}_{secondary_index}")?,
            _ if secondary_index != 0 => write!(f, " JUMP +{secondary_index}")?,
            _ => write!(f, " <invalid secondary operand>")?,
        }

        Ok(())
    }
}
