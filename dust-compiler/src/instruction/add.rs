use std::fmt::{self, Display, Formatter};

use crate::instruction::{Address, Instruction, InstructionBuilder, OperandType, Operation};

pub struct Add {
    pub destination: u16,
    pub operand_type: OperandType,
    pub left_address: Address,
    pub right_address: Address,
}

impl From<Instruction> for Add {
    fn from(instruction: Instruction) -> Self {
        Add {
            destination: instruction.a_field(),
            operand_type: instruction.operand_type(),
            left_address: instruction.b_address(),
            right_address: instruction.c_address(),
        }
    }
}

impl From<Add> for Instruction {
    fn from(add: Add) -> Self {
        let Add {
            destination,
            operand_type,
            left_address,
            right_address,
        } = add;

        InstructionBuilder::new(Operation::ADD)
            .a_field(destination)
            .operand_type(operand_type)
            .b_address(left_address)
            .c_address(right_address)
            .build()
    }
}

impl Display for Add {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let Add {
            destination,
            operand_type,
            left_address,
            right_address,
        } = *self;

        write!(
            f,
            "reg_{destination}: {operand_type} = {left_address} + {right_address}"
        )
    }
}
