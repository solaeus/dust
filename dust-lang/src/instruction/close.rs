use std::fmt::{self, Display, Formatter};

use super::{Address, Instruction, InstructionFields, OperandType, Operation};

pub struct Close {
    pub from: Address,
    pub to: Address,
    pub r#type: OperandType,
}

impl From<&Instruction> for Close {
    fn from(instruction: &Instruction) -> Self {
        let from = instruction.b_address();
        let to = instruction.c_address();
        let r#type = instruction.operand_type();

        Close { from, to, r#type }
    }
}

impl From<Close> for Instruction {
    fn from(close: Close) -> Self {
        let operation = Operation::CLOSE;
        let Address {
            index: b_field,
            memory: b_memory_kind,
        } = close.from;
        let Address {
            index: c_field,
            memory: c_memory_kind,
        } = close.to;
        let operand_type = close.r#type;

        InstructionFields {
            operation,
            b_field,
            b_memory_kind,
            c_field,
            c_memory_kind,
            operand_type,
            ..Default::default()
        }
        .build()
    }
}

impl Display for Close {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let Close { from, to, r#type } = self;

        from.display(f, *r#type)?;
        write!(f, "..=")?;
        to.display(f, *r#type)
    }
}
