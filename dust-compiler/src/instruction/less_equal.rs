use std::fmt::{self, Display, Formatter};

use crate::instruction::{Address, Instruction, InstructionBuilder, OperandType, Operation};

pub struct LessEqual {
    pub comparator: bool,
    pub operand_type: OperandType,
    pub left_address: Address,
    pub right_address: Address,
}

impl From<Instruction> for LessEqual {
    fn from(instruction: Instruction) -> Self {
        LessEqual {
            comparator: instruction.a_field() != 0,
            operand_type: instruction.operand_type(),
            left_address: instruction.b_address(),
            right_address: instruction.c_address(),
        }
    }
}

impl From<LessEqual> for Instruction {
    fn from(less_equal: LessEqual) -> Self {
        let LessEqual {
            comparator,
            operand_type,
            left_address,
            right_address,
        } = less_equal;

        InstructionBuilder::new(Operation::LESS_EQUAL)
            .a_field(if comparator { 1 } else { 0 })
            .operand_type(operand_type)
            .b_address(left_address)
            .c_address(right_address)
            .build()
    }
}

impl Display for LessEqual {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let LessEqual {
            comparator,
            operand_type: _,
            left_address,
            right_address,
        } = *self;
        let operator = if comparator { "≤" } else { ">" };

        write!(f, "if {left_address} {operator} {right_address} jump +1")
    }
}
