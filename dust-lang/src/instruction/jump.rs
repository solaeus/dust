use std::fmt::{self, Display, Formatter};

use crate::instruction::{Instruction, InstructionBuilder, Operation};

pub struct Jump {
    pub offset: u16,
    pub is_positive: bool,
    pub drop_list_start: u16,
    pub drop_list_end: u16,
}

impl From<&Instruction> for Jump {
    fn from(instruction: &Instruction) -> Self {
        Jump {
            offset: instruction.a_field(),
            is_positive: instruction.e_field(),
            drop_list_start: instruction.b_field(),
            drop_list_end: instruction.c_field(),
        }
    }
}

impl From<Jump> for Instruction {
    fn from(jump: Jump) -> Self {
        let Jump {
            offset,
            is_positive,
            drop_list_start,
            drop_list_end,
        } = jump;

        InstructionBuilder::new(Operation::JUMP)
            .a_field(offset)
            .e_field(is_positive)
            .b_field(drop_list_start)
            .c_field(drop_list_end)
            .build()
    }
}

impl Display for Jump {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let Jump {
            offset,
            is_positive,
            drop_list_start,
            drop_list_end,
        } = self;
        let sign = if *is_positive { "+" } else { "-" };

        write!(f, "jump {sign}{offset}")?;

        if drop_list_end > drop_list_start {
            write!(f, " drop {drop_list_start}..{drop_list_end}")
        } else {
            Ok(())
        }
    }
}
