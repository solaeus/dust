use std::fmt::{self, Display, Formatter};

use super::{Address, Instruction, InstructionFields, Operation};

pub struct Return {
    pub operand: Address,
}

impl From<&Instruction> for Return {
    fn from(instruction: &Instruction) -> Self {
        let operand = instruction.b_address();

        Return { operand }
    }
}

impl From<Return> for Instruction {
    fn from(r#return: Return) -> Self {
        let operation = Operation::RETURN;
        let Address {
            index: b_field,
            memory: b_memory_kind,
        } = r#return.operand;

        InstructionFields {
            operation,
            b_field,
            b_memory_kind,
            ..Default::default()
        }
        .build()
    }
}

impl Display for Return {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let Return { operand } = self;

        write!(f, "return {operand}")
    }
}
