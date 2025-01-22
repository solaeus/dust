use std::fmt::{self, Display, Formatter};

use super::{Instruction, Operand, Operation, TwoOperandLayout, TypeCode};

pub struct Less {
    pub comparator: bool,
    pub left: Operand,
    pub left_type: TypeCode,
    pub right: Operand,
    pub right_type: TypeCode,
}

impl From<Instruction> for Less {
    fn from(instruction: Instruction) -> Self {
        let comparator = instruction.d_field();
        let (left, right) = instruction.b_and_c_as_operands();
        let left_type = instruction.b_type();
        let right_type = instruction.c_type();

        Less {
            comparator,
            left,
            left_type,
            right,
            right_type,
        }
    }
}

impl From<Less> for Instruction {
    fn from(less: Less) -> Self {
        let operation = Operation::LESS;
        let (b_field, b_is_constant) = less.left.as_index_and_constant_flag();
        let (c_field, c_is_constant) = less.right.as_index_and_constant_flag();
        let d_field = less.comparator;
        let b_type = less.left_type;
        let c_type = less.right_type;

        TwoOperandLayout {
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

impl Display for Less {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let Less {
            comparator,
            left,
            left_type,
            right,
            right_type,
        } = self;
        let operator = if *comparator { "<" } else { "≥" };
        let left_name = match *left {
            Operand::Register(_) => match *left_type {
                TypeCode::INTEGER => "R_INT",
                TypeCode::FLOAT => "R_FLOAT",
                unknown => unknown.panic_from_unknown_code(),
            },
            Operand::Constant(_) => match *left_type {
                TypeCode::INTEGER => "C_INT",
                TypeCode::FLOAT => "C_FLOAT",
                unknown => unknown.panic_from_unknown_code(),
            },
        };
        let right_name = match *right {
            Operand::Register(_) => match *right_type {
                TypeCode::INTEGER => "R_INT",
                TypeCode::FLOAT => "R_FLOAT",
                unknown => unknown.panic_from_unknown_code(),
            },
            Operand::Constant(_) => match *right_type {
                TypeCode::INTEGER => "C_INT",
                TypeCode::FLOAT => "C_FLOAT",
                unknown => unknown.panic_from_unknown_code(),
            },
        };

        write!(
            f,
            "if {left_name}_{left_index} {operator} {right_name}_{right_index} {{ JUMP +1 }}",
            left_index = left.index(),
            right_index = right.index(),
        )
    }
}
