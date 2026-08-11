use std::fmt::{self, Display, Formatter};

use crate::instruction::{Address, Instruction, InstructionBuilder, OperandType, Operation};

pub struct Get {
    pub destination: u16,
    pub operand_type: OperandType,
    pub base_register: u16,
    pub index: Address,
}

impl From<Instruction> for Get {
    fn from(instruction: Instruction) -> Self {
        Get {
            destination: instruction.a_field() as u16,
            operand_type: instruction.operand_type(),
            base_register: instruction.b_field() as u16,
            index: instruction.c_address(),
        }
    }
}

impl From<Get> for Instruction {
    fn from(get_index: Get) -> Self {
        let Get {
            destination,
            operand_type,
            base_register,
            index,
        } = get_index;

        InstructionBuilder::new(Operation::GET)
            .a_field(destination)
            .operand_type(operand_type)
            .b_field(base_register)
            .c_address(index)
            .build()
    }
}

impl Display for Get {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let Get {
            destination,
            operand_type,
            base_register,
            index,
        } = *self;

        write!(
            f,
            "reg_{destination}: {operand_type} = reg_{base_register}[{index}]"
        )
    }
}
