use std::fmt::{self, Display, Formatter};

use crate::instruction::{Instruction, InstructionBuilder, OperandType, Operation};

pub struct NewList {
    pub destination: u16,
    pub element_type: OperandType,
    pub initial_length: u32,
}

impl From<&Instruction> for NewList {
    fn from(instruction: &Instruction) -> Self {
        NewList {
            destination: instruction.a_field(),
            element_type: instruction.operand_type(),
            initial_length: instruction.bc_field(),
        }
    }
}

impl From<NewList> for Instruction {
    fn from(list: NewList) -> Self {
        let NewList {
            destination,
            element_type,
            initial_length,
        } = list;

        InstructionBuilder::new(Operation::NEW_LIST)
            .a_field(destination)
            .operand_type(element_type)
            .bc_field(initial_length)
            .build()
    }
}

impl Display for NewList {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let NewList {
            destination,
            element_type,
            initial_length,
        } = self;

        write!(f, "reg_{destination} = [{element_type}; {initial_length}]")
    }
}
