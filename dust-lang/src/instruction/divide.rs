use std::fmt::{self, Display, Formatter};

use super::{Instruction, InstructionFields, Operand, Operation, TypeCode};

pub struct Divide {
    pub destination: u16,
    pub left: Operand,
    pub right: Operand,
}

impl From<Instruction> for Divide {
    fn from(instruction: Instruction) -> Self {
        let destination = instruction.a_field();
        let (left, right) = instruction.b_and_c_as_operands();

        Divide {
            destination,
            left,
            right,
        }
    }
}

impl From<Divide> for Instruction {
    fn from(divide: Divide) -> Self {
        let operation = Operation::DIVIDE;
        let a_field = divide.destination;
        let (b_field, b_is_constant) = divide.left.as_index_and_constant_flag();
        let (c_field, c_is_constant) = divide.right.as_index_and_constant_flag();
        let b_type = divide.left.as_type();
        let c_type = divide.right.as_type();

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

impl Display for Divide {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let Divide {
            destination,
            left,
            right,
        } = self;

        match left.as_type() {
            TypeCode::BOOLEAN => write!(f, "R_BOOL_{destination}")?,
            TypeCode::BYTE => write!(f, "R_BYTE_{destination}")?,
            TypeCode::CHARACTER => write!(f, "R_STR_{destination}")?,
            TypeCode::FLOAT => write!(f, "R_FLOAT_{destination}")?,
            TypeCode::INTEGER => write!(f, "R_INT_{destination}")?,
            TypeCode::STRING => write!(f, "R_STR_{destination}")?,
            _ => todo!(),
        }

        write!(f, " = {left} ÷ {right}",)
    }
}
