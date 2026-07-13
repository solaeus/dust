use std::fmt::{self, Display, Formatter};

use crate::instruction::{Address, Instruction, InstructionBuilder, OperandType, Operation};

pub struct Power {
    pub destination: u16,
    pub operand_type: OperandType,
    pub base_address: Address,
    pub exponent_address: Address,
}

impl From<Instruction> for Power {
    fn from(instruction: Instruction) -> Self {
        Power {
            destination: instruction.a_field(),
            operand_type: instruction.operand_type(),
            base_address: instruction.b_address(),
            exponent_address: instruction.c_address(),
        }
    }
}

impl From<Power> for Instruction {
    fn from(power: Power) -> Self {
        let Power {
            destination,
            operand_type,
            base_address,
            exponent_address,
        } = power;

        InstructionBuilder::new(Operation::POWER)
            .a_field(destination)
            .operand_type(operand_type)
            .b_address(base_address)
            .c_address(exponent_address)
            .build()
    }
}

impl Display for Power {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let Power {
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
