use std::fmt::{self, Display, Formatter};

use super::{Instruction, InstructionFields, Operand, Operation, TypeCode};

pub struct Add {
    pub destination: u16,
    pub left: Operand,
    pub left_type: TypeCode,
    pub right: Operand,
    pub right_type: TypeCode,
}

impl From<Instruction> for Add {
    fn from(instruction: Instruction) -> Self {
        let destination = instruction.a_field();
        let (left, right) = instruction.b_and_c_as_operands();
        let left_type = instruction.b_type();
        let right_type = instruction.c_type();

        Add {
            destination,
            left,
            left_type,
            right,
            right_type,
        }
    }
}

impl From<Add> for Instruction {
    fn from(add: Add) -> Self {
        let operation = Operation::ADD;
        let a_field = add.destination;
        let (b_field, b_is_constant) = add.left.as_index_and_constant_flag();
        let (c_field, c_is_constant) = add.right.as_index_and_constant_flag();
        let b_type = add.left_type;
        let c_type = add.right_type;

        InstructionFields {
            operation,
            a_field,
            b_field,
            c_field,
            b_is_constant,
            c_is_constant,
            b_type,
            c_type,
            ..Default::default()
        }
        .build()
    }
}

impl Display for Add {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let Add {
            destination,
            left,
            left_type,
            right,
            right_type: _,
        } = self;

        match *left_type {
            TypeCode::BOOLEAN => write!(f, "R_BOOL_{destination}")?,
            TypeCode::BYTE => write!(f, "R_BYTE_{destination}")?,
            TypeCode::CHARACTER => write!(f, "R_CHAR_{destination}")?,
            TypeCode::FLOAT => write!(f, "R_FLOAT_{destination}")?,
            TypeCode::INTEGER => write!(f, "R_INT_{destination}")?,
            TypeCode::STRING => write!(f, "R_STR_{destination}")?,
            _ => todo!(),
        }

        write!(f, " = {left} + {right}",)
    }
}
