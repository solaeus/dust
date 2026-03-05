use std::fmt::{self, Display, Formatter};

use crate::instruction::{Instruction, InstructionBuilder, MemoryKind, OperandType, Operation};

pub struct Return {
    pub returns_value: bool,
    pub operand_type: OperandType,
    pub operand_memory: MemoryKind,
    pub operand_index: u16,
    pub additional_registers: u16,
}

impl From<&Instruction> for Return {
    fn from(instruction: &Instruction) -> Self {
        Return {
            returns_value: instruction.a_field() != 0,
            operand_type: instruction.operand_type(),
            operand_memory: instruction.b_memory(),
            operand_index: instruction.b_field(),
            additional_registers: instruction.c_field(),
        }
    }
}

impl From<Return> for Instruction {
    fn from(r#return: Return) -> Self {
        let Return {
            returns_value,
            operand_type,
            operand_memory,
            operand_index,
            additional_registers,
        } = r#return;

        InstructionBuilder::new(Operation::RETURN)
            .operand_type(operand_type)
            .a_field(returns_value as u16)
            .b_memory(operand_memory)
            .b_field(operand_index)
            .c_field(additional_registers)
            .build()
    }
}

impl Display for Return {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let Return {
            returns_value,
            operand_type,
            operand_memory,
            operand_index,
            additional_registers,
        } = *self;
        let operand_memory = operand_memory.as_string(operand_type);

        write!(f, "return")?;

        if returns_value {
            write!(f, " {operand_memory}_{operand_index}")?;

            if additional_registers > 0 {
                let last_register = operand_index + additional_registers;

                write!(f, "..=reg_{last_register}")?;
            }
        }

        Ok(())
    }
}
