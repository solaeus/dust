use std::fmt::{self, Display, Formatter};

use crate::instruction::{Address, Instruction, InstructionBuilder, OperandType, Operation};

pub struct Negate {
    pub destination: u16,
    pub operand_type: OperandType,
    pub operand: Address,
}

impl From<Instruction> for Negate {
    fn from(instruction: Instruction) -> Self {
        Negate {
            destination: instruction.a_field() as u16,
            operand_type: instruction.operand_type(),
            operand: instruction.b_address(),
        }
    }
}

impl From<Negate> for Instruction {
    fn from(negate: Negate) -> Self {
        let Negate {
            destination,
            operand_type,
            operand,
        } = negate;

        InstructionBuilder::new(Operation::NEGATE)
            .a_field(destination)
            .operand_type(operand_type)
            .b_address(operand)
            .build()
    }
}

impl Display for Negate {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let Negate {
            destination,
            operand_type,
            operand,
        } = *self;
        let operator = match operand_type {
            OperandType::BOOLEAN => "!",
            _ => "-",
        };

        write!(f, "reg_{destination}: {operand_type} = {operator}{operand}")
    }
}
