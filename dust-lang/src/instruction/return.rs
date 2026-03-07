use std::fmt::{self, Display, Formatter};

use crate::instruction::{Instruction, InstructionBuilder, Operation};

pub struct Return {
    pub returns_value: bool,
    pub argument_count: u16,
}

impl From<&Instruction> for Return {
    fn from(instruction: &Instruction) -> Self {
        Return {
            returns_value: instruction.a_field() != 0,
            argument_count: instruction.c_field(),
        }
    }
}

impl From<Return> for Instruction {
    fn from(r#return: Return) -> Self {
        let Return {
            returns_value,
            argument_count,
        } = r#return;

        InstructionBuilder::new(Operation::RETURN)
            .a_field(returns_value as u16)
            .c_field(argument_count)
            .build()
    }
}

impl Display for Return {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let Return {
            returns_value,
            argument_count,
        } = *self;

        write!(f, "return")?;

        if returns_value {
            write!(f, " reg_0..reg_{}", argument_count)?;
        }

        Ok(())
    }
}
