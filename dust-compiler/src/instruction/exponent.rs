use std::fmt::{self, Display, Formatter};

use crate::instruction::{Address, Instruction, InstructionBuilder, OperandType, Operation};

pub struct Exponent {
    pub destination: u16,
    pub operand_type: OperandType,
    pub base_address: Address,
    pub exponent_address: Address,
}

impl From<Instruction> for Exponent {
    fn from(instruction: Instruction) -> Self {
        Exponent {
            destination: instruction.a_field(),
            operand_type: instruction.operand_type(),
            base_address: instruction.b_address(),
            exponent_address: instruction.c_address(),
        }
    }
}

impl From<Exponent> for Instruction {
    fn from(power: Exponent) -> Self {
        let Exponent {
            destination,
            operand_type,
            base_address,
            exponent_address,
        } = power;

        InstructionBuilder::new(Operation::EXPONENT)
            .a_field(destination)
            .operand_type(operand_type)
            .b_address(base_address)
            .c_address(exponent_address)
            .build()
    }
}

impl Display for Exponent {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let Exponent {
            destination,
            operand_type,
            base_address,
            exponent_address,
        } = *self;

        write!(
            f,
            "reg_{destination}: {operand_type} = {base_address} ^ {exponent_address}"
        )
    }
}
