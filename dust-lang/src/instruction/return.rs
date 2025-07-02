use std::fmt::{self, Display, Formatter};

use super::{Address, Instruction, InstructionFields, OperandType, Operation};

pub struct Return {
    pub should_return_value: bool,
    pub return_value_address: Address,
    pub r#type: OperandType,
}

impl From<&Instruction> for Return {
    fn from(instruction: &Instruction) -> Self {
        let should_return_value = instruction.a_field() != 0;
        let return_value_address = instruction.b_address();
        let r#type = instruction.operand_type();

        Return {
            should_return_value,
            return_value_address,
            r#type,
        }
    }
}

impl From<Return> for Instruction {
    fn from(r#return: Return) -> Self {
        let operation = Operation::RETURN;
        let a_field = r#return.should_return_value as usize;
        let Address {
            index: b_field,
            memory: b_memory_kind,
        } = r#return.return_value_address;
        let operand_type = r#return.r#type;

        InstructionFields {
            operation,
            a_field,
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
        let Return {
            should_return_value,
            return_value_address,
            ..
        } = self;

        if *should_return_value {
            write!(f, "RETURN {return_value_address}")?;
        } else {
            write!(f, "RETURN")?;
        }

        Ok(())
    }
}
