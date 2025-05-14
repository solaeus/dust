use std::fmt::{self, Display, Formatter};

use crate::{Instruction, Operation};

use super::{Address, InstructionFields};

pub struct Return {
    pub should_return_value: bool,
    pub return_address: Address,
}

impl From<&Instruction> for Return {
    fn from(instruction: &Instruction) -> Self {
        let should_return_value = instruction.a_field() != 0;
        let return_address = instruction.b_address();

        Return {
            should_return_value,
            return_address,
        }
    }
}

impl From<Return> for Instruction {
    fn from(r#return: Return) -> Self {
        let operation = Operation::RETURN;
        let a_field = r#return.should_return_value as u16;
        let Address {
            index: b_field,
            kind: b_kind,
        } = r#return.return_address;

        InstructionFields {
            operation,
            a_field,
            b_field,
            b_kind,
            ..Default::default()
        }
        .build()
    }
}

impl Display for Return {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let Return {
            should_return_value,
            return_address,
        } = self;
        write!(f, "RETURN")?;

        if *should_return_value {
            write!(f, " {return_address}")?;
        }

        Ok(())
    }
}
