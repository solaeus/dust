use std::fmt::{self, Display, Formatter};

use crate::instruction::{Instruction, InstructionBuilder, Operation};

pub struct Return {
    pub returns_value: bool,
    pub arguments_start: u16,
    pub argument_count: u16,
}

impl From<&Instruction> for Return {
    fn from(instruction: &Instruction) -> Self {
        Return {
            returns_value: instruction.a_field() != 0,
            arguments_start: instruction.b_field(),
            argument_count: instruction.c_field(),
        }
    }
}

impl From<Return> for Instruction {
    fn from(r#return: Return) -> Self {
        let Return {
            returns_value,
            arguments_start,
            argument_count,
        } = r#return;

        InstructionBuilder::new(Operation::RETURN)
            .a_field(returns_value as u16)
            .b_field(arguments_start)
            .c_field(argument_count)
            .build()
    }
}

impl Display for Return {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let Return {
            returns_value,
            arguments_start,
            argument_count,
        } = *self;

        write!(f, "return")?;

        if returns_value {
            let arguments_end = arguments_start + argument_count;

            write!(f, " args_{arguments_start}..=args_{arguments_end}")?;
        }

        Ok(())
    }
}
