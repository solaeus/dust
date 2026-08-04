use std::fmt::{self, Display, Formatter};

use crate::instruction::{Address, Instruction, InstructionBuilder, OperandType, Operation};

pub struct Set {
    pub base: u16,
    pub operand_type: OperandType,
    pub index: Address,
    pub source: Address,
}

impl From<Instruction> for Set {
    fn from(instruction: Instruction) -> Self {
        Set {
            base: instruction.a_field(),
            operand_type: instruction.operand_type(),
            index: instruction.b_address(),
            source: instruction.c_address(),
        }
    }
}

impl From<Set> for Instruction {
    fn from(set_index: Set) -> Self {
        let Set {
            base,
            operand_type,
            index,
            source,
        } = set_index;

        InstructionBuilder::new(Operation::SET)
            .a_field(base)
            .operand_type(operand_type)
            .b_address(index)
            .c_address(source)
            .build()
    }
}

impl Display for Set {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let Set {
            base,
            operand_type,
            index,
            source,
        } = *self;

        write!(f, "reg_{base}[{index}]: {operand_type} = {source}",)
    }
}
