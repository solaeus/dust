use std::fmt::{self, Display, Formatter};

use super::{Address, Instruction, InstructionFields, OperandType, Operation};

pub struct Return {
    pub operand: Address,
    pub r#type: OperandType,
}

impl From<&Instruction> for Return {
    fn from(instruction: &Instruction) -> Self {
        let operand = instruction.b_address();
        let r#type = instruction.operand_type();

        Return { operand, r#type }
    }
}

impl From<Return> for Instruction {
    fn from(r#return: Return) -> Self {
        let operation = Operation::RETURN;
        let Address {
            index: b_field,
            memory: b_memory_kind,
        } = r#return.operand;
        let operand_type = r#return.r#type;

        InstructionFields {
            operation,
            b_field,
            b_memory_kind,
            operand_type,
            ..Default::default()
        }
        .build()
    }
}

impl Display for Return {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let Return { operand, r#type } = self;

        if *r#type == OperandType::NONE {
            write!(f, "return")
        } else {
            write!(f, "return ")?;
            operand.display(f, *r#type)
        }
    }
}
