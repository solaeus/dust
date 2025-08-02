use std::fmt::{self, Display, Formatter};

use super::{Address, Instruction, InstructionFields, OperandType, Operation};

pub struct Less {
    pub comparator: usize,
    pub left: Address,
    pub right: Address,
    pub r#type: OperandType,
}

impl From<Instruction> for Less {
    fn from(instruction: Instruction) -> Self {
        let comparator = instruction.a_field();
        let left = instruction.b_address();
        let right = instruction.c_address();
        let r#type = instruction.operand_type();

        Less {
            comparator,
            left,
            right,
            r#type,
        }
    }
}

impl From<Less> for Instruction {
    fn from(less: Less) -> Self {
        let operation = Operation::LESS;
        let a_field = less.comparator;
        let Address {
            index: b_field,
            memory: b_memory_kind,
        } = less.left;
        let Address {
            index: c_field,
            memory: c_memory_kind,
        } = less.right;
        let operand_type = less.r#type;

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

impl Display for Less {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let Less {
            comparator,
            left,
            right,
            r#type,
        } = self;
        let operator = if *comparator != 0 { "<" } else { "≥" };

        write!(f, "if ")?;
        left.display(f, *r#type)?;
        write!(f, " {operator} ")?;
        right.display(f, *r#type)?;
        write!(f, " {{ JUMP +1 }}")
    }
}
