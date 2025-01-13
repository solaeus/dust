use std::fmt::{self, Display, Formatter};

use super::{Instruction, InstructionBuilder, Operand, Operation, TypeCode};

pub struct Equal {
    pub comparator: bool,
    pub left: Operand,
    pub left_type: TypeCode,
    pub right: Operand,
    pub right_type: TypeCode,
}

impl From<Instruction> for Equal {
    fn from(instruction: Instruction) -> Self {
        let comparator = instruction.d_field();
        let (left, right) = instruction.b_and_c_as_operands();
        let left_type = instruction.b_type();
        let right_type = instruction.c_type();

        Equal {
            comparator,
            left,
            left_type,
            right,
            right_type,
        }
    }
}

impl From<Equal> for Instruction {
    fn from(equal_bool: Equal) -> Self {
        let operation = Operation::EQUAL;
        let (b_field, b_is_constant) = equal_bool.left.as_index_and_constant_flag();
        let (c_field, c_is_constant) = equal_bool.right.as_index_and_constant_flag();
        let d_field = equal_bool.comparator;
        let b_type = equal_bool.left_type;
        let c_type = equal_bool.right_type;

        InstructionBuilder {
            operation,
            b_field,
            c_field,
            d_field,
            b_is_constant,
            c_is_constant,
            b_type,
            c_type,
            ..Default::default()
        }
        .build()
    }
}

impl Display for Equal {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let Equal {
            comparator,
            left,
            left_type,
            right,
            right_type,
        } = self;
        let operator = if *comparator { "==" } else { "≠" };

        write!(
            f,
            "if {left}({left_type}) {operator} {right}({right_type}) {{ JUMP +1 }}"
        )
    }
}
