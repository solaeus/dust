use std::fmt::{self, Display, Formatter};

use crate::instruction::{Address, Instruction, InstructionBuilder, OperandType, Operation};

pub struct Equal {
    pub comparator: bool,
    pub operand_type: OperandType,
    pub left_address: Address,
    pub right_address: Address,
}

impl From<Instruction> for Equal {
    fn from(instruction: Instruction) -> Self {
        Equal {
            comparator: instruction.a_field() != 0,
            operand_type: instruction.operand_type(),
            left_address: instruction.b_address(),
            right_address: instruction.c_address(),
        }
    }
}

impl From<Equal> for Instruction {
    fn from(equal: Equal) -> Self {
        let Equal {
            comparator,
            operand_type,
            left_address,
            right_address,
        } = equal;

        InstructionBuilder::new(Operation::EQUAL)
            .a_field(comparator as u16)
            .operand_type(operand_type)
            .b_address(left_address)
            .c_address(right_address)
            .build()
    }
}

impl Display for Equal {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let Equal {
            comparator,
            operand_type: _,
            left_address,
            right_address,
        } = *self;
        let operator = if comparator { "==" } else { "≠" };

        write!(f, "if {left_address} {operator} {right_address} jump +1")
    }
}
