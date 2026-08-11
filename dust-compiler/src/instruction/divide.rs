use std::fmt::{self, Display, Formatter};

use crate::instruction::{Address, Instruction, InstructionBuilder, OperandType, Operation};

pub struct Divide {
    pub destination: u16,
    pub operand_type: OperandType,
    pub left_address: Address,
    pub right_address: Address,
}

impl From<Instruction> for Divide {
    fn from(instruction: Instruction) -> Self {
        Divide {
            destination: instruction.a_field() as u16,
            operand_type: instruction.operand_type(),
            left_address: instruction.b_address(),
            right_address: instruction.c_address(),
        }
    }
}

impl From<Divide> for Instruction {
    fn from(divide: Divide) -> Self {
        let Divide {
            destination,
            operand_type,
            left_address,
            right_address,
        } = divide;

        InstructionBuilder::new(Operation::DIVIDE)
            .a_field(destination)
            .operand_type(operand_type)
            .b_address(left_address)
            .c_address(right_address)
            .build()
    }
}

impl Display for Divide {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let Divide {
            destination,
            operand_type,
            left_address,
            right_address,
        } = *self;

        write!(
            f,
            "reg_{destination}: {operand_type} = {left_address} / {right_address}"
        )
    }
}
