use std::fmt::{self, Display, Formatter};

use crate::instruction::{Instruction, InstructionBuilder, MemoryKind, Operation};

pub struct Jump {
    pub offset: u16,
    pub is_positive: bool,
    pub drop_register_start: u16,
    pub drop_list_end: u16,
}

impl From<Instruction> for Jump {
    fn from(instruction: Instruction) -> Self {
        Jump {
            offset: instruction.a_field() as u16,
            is_positive: instruction.b_memory().0 != 0,
            drop_register_start: instruction.b_field() as u16,
            drop_list_end: instruction.c_field() as u16,
        }
    }
}

impl From<Jump> for Instruction {
    fn from(jump: Jump) -> Self {
        let Jump {
            offset,
            is_positive,
            drop_register_start,
            drop_list_end,
        } = jump;

        InstructionBuilder::new(Operation::JUMP)
            .a_field(offset)
            .b_memory(MemoryKind(is_positive as u8))
            .b_field(drop_register_start)
            .c_field(drop_list_end)
            .build()
    }
}

impl Display for Jump {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let Jump {
            offset,
            is_positive,
            drop_register_start,
            drop_list_end,
        } = self;
        let sign = if *is_positive { "+" } else { "-" };

        write!(f, "jump {sign}{offset}")?;

        if drop_list_end > drop_register_start {
            write!(f, " drop {drop_register_start}..{drop_list_end}")
        } else {
            Ok(())
        }
    }
}
