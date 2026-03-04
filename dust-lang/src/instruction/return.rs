use std::fmt::{self, Display, Formatter};

use crate::instruction::SmallType;

use super::{Address, Instruction, InstructionFields, Operation};

pub struct Return {
    pub operand_type: SmallType,
    pub operand: Address,
    pub secondary_index: u16,
}

impl From<&Instruction> for Return {
    fn from(instruction: &Instruction) -> Self {
        let operand_type = instruction.operand_type();
        let operand = instruction.b_address();
        let secondary_index = instruction.c_field();

        Return {
            operand_type,
            operand,
            secondary_index,
        }
    }
}

impl From<Return> for Instruction {
    fn from(r#return: Return) -> Self {
        let operation = Operation::RETURN;
        let Address {
            index: b_field,
            memory: b_memory_kind,
        } = r#return.operand;
        let c_field = r#return.secondary_index;

        InstructionFields {
            operation,
            b_field,
            b_memory_kind,
            c_field,
            ..Default::default()
        }
        .build()
    }
}

impl Display for Return {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let Return {
            operand_type,
            operand,
            secondary_index,
        } = *self;

        write!(f, "return")?;

        if operand_type != SmallType::UNIT {
            write!(f, " {operand}")?;
        }

        let memory_kind = operand.memory;

        match operand_type {
            SmallType::U_128 | SmallType::I_128 => write!(f, " & {memory_kind}_{secondary_index}")?,
            SmallType::STRUCT => write!(f, "..={memory_kind}_{secondary_index}")?,
            _ if secondary_index != 0 => write!(f, " JUMP +{secondary_index}")?,
            _ => write!(f, " <invalid secondary operand>")?,
        }

        Ok(())
    }
}
