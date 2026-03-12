use std::fmt::{self, Display, Formatter};

use crate::instruction::{Instruction, InstructionBuilder, Operation};

pub struct Return;

impl From<&Instruction> for Return {
    fn from(_: &Instruction) -> Self {
        Return
    }
}

impl From<Return> for Instruction {
    fn from(_: Return) -> Self {
        InstructionBuilder::new(Operation::RETURN).build()
    }
}

impl Display for Return {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "return")
    }
}
