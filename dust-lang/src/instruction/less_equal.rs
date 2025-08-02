use std::fmt::{self, Display, Formatter};

use super::{Address, Instruction, InstructionFields, OperandType, Operation};

pub struct LessEqual {
    pub comparator: usize,
    pub left: Address,
    pub right: Address,
    pub r#type: OperandType,
}

impl From<Instruction> for LessEqual {
    fn from(instruction: Instruction) -> Self {
        let comparator = instruction.a_field();
        let left = instruction.b_address();
        let right = instruction.c_address();
        let r#type = instruction.operand_type();

        LessEqual {
            comparator,
            left,
            right,
            r#type,
        }
    }
}

impl From<LessEqual> for Instruction {
    fn from(less_equal: LessEqual) -> Self {
        let operation = Operation::LESS_EQUAL;
        let a_field = less_equal.comparator;
        let Address {
            index: b_field,
            memory: b_memory_kind,
        } = less_equal.left;
        let Address {
            index: c_field,
            memory: c_memory_kind,
        } = less_equal.right;
        let operand_type = less_equal.r#type;

        InstructionFields {
            operation,
            a_field,
            b_field,
            b_memory_kind,
            c_field,
            c_memory_kind,
            operand_type,
            ..Default::default()
        }
        .build()
    }
}

impl Display for LessEqual {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let LessEqual {
            comparator,
            left,
            right,
            r#type,
        } = self;
        let operator = if *comparator != 0 { "≤" } else { ">" };

        write!(f, "if ")?;
        left.display(f, *r#type)?;
        write!(f, " {operator} ")?;
        right.display(f, *r#type)?;
        write!(f, " {{ JUMP +1 }}")
    }
}
