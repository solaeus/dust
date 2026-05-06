use std::fmt::{self, Display, Formatter};

use crate::instruction::{Address, Instruction, InstructionBuilder, OperandType, Operation};

pub struct Subtract {
    pub destination: u16,
    pub operand_type: OperandType,
    pub left_address: Address,
    pub right_address: Address,
}

impl From<&Instruction> for Subtract {
    fn from(instruction: &Instruction) -> Self {
        Subtract {
            destination: instruction.a_field(),
            operand_type: instruction.operand_type(),
            left_address: instruction.b_address(),
            right_address: instruction.c_address(),
        }
    }
}

impl From<Subtract> for Instruction {
    fn from(subtract: Subtract) -> Self {
        let Subtract {
            destination,
            operand_type,
            left_address,
            right_address,
        } = subtract;

        InstructionBuilder::new(Operation::SUBTRACT)
            .a_field(destination)
            .operand_type(operand_type)
            .b_address(left_address)
            .c_address(right_address)
            .build()
    }
}

impl Display for Subtract {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let Subtract {
            destination,
            operand_type,
            left_address,
            right_address,
        } = *self;

        write!(
            f,
            "reg_{destination}: {operand_type} = {left_address} - {right_address}"
        )
    }
}
