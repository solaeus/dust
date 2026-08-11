use std::fmt::{self, Display, Formatter};

use crate::instruction::{Address, Instruction, InstructionBuilder, OperandType, Operation};

pub struct Multiply {
    pub destination: u16,
    pub operand_type: OperandType,
    pub left_address: Address,
    pub right_address: Address,
}

impl From<Instruction> for Multiply {
    fn from(instruction: Instruction) -> Self {
        Multiply {
            destination: instruction.a_field() as u16,
            operand_type: instruction.operand_type(),
            left_address: instruction.b_address(),
            right_address: instruction.c_address(),
        }
    }
}

impl From<Multiply> for Instruction {
    fn from(multiply: Multiply) -> Self {
        let Multiply {
            destination,
            operand_type,
            left_address,
            right_address,
        } = multiply;

        InstructionBuilder::new(Operation::MULTIPLY)
            .a_field(destination)
            .operand_type(operand_type)
            .b_address(left_address)
            .c_address(right_address)
            .build()
    }
}

impl Display for Multiply {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let Multiply {
            destination,
            operand_type,
            left_address,
            right_address,
        } = *self;

        write!(
            f,
            "reg_{destination}: {operand_type} = {left_address} * {right_address}"
        )
    }
}
