use std::fmt::{self, Display, Formatter};

use super::{Address, Instruction, InstructionFields, OperandType, Operation};

pub struct Load {
    pub destination: Address,
    pub operand: Address,
    pub r#type: OperandType,
    pub jump_next: bool,
}

impl From<Instruction> for Load {
    fn from(instruction: Instruction) -> Self {
        let destination = instruction.destination();
        let operand = instruction.b_address();
        let r#type = instruction.operand_type();
        let jump_next = instruction.c_field() != 0;

        Load {
            destination,
            operand,
            r#type,
            jump_next,
        }
    }
}

impl From<Load> for Instruction {
    fn from(load: Load) -> Self {
        let Address {
            index: a_field,
            memory: a_memory_kind,
        } = load.destination;
        let Address {
            index: b_field,
            memory: b_memory_kind,
        } = load.operand;
        let c_field = load.jump_next as usize;
        let operand_type = load.r#type;

        InstructionFields {
            operation: Operation::LOAD,
            a_field,
            a_memory_kind,
            b_field,
            b_memory_kind,
            c_field,
            operand_type,
            ..Default::default()
        }
        .build()
    }
}

impl Display for Load {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let Load {
            destination,
            operand,
            jump_next,
            ..
        } = self;

        write!(f, "{destination} = {operand}")?;

        if *jump_next {
            write!(f, " JUMP +1")?;
        }

        Ok(())
    }
}
