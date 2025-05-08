use std::fmt::{self, Display, Formatter};

use super::{Instruction, InstructionFields, Operand, Operation, TypeCode};

pub struct Modulo {
    pub destination: u16,
    pub left: Operand,
    pub right: Operand,
}

impl From<Instruction> for Modulo {
    fn from(instruction: Instruction) -> Self {
        let destination = instruction.a_field();
        let (left, right) = instruction.b_and_c_as_operands();

        Modulo {
            destination,
            left,
            right,
        }
    }
}

impl From<Modulo> for Instruction {
    fn from(modulo: Modulo) -> Self {
        let operation = Operation::MODULO;
        let a_field = modulo.destination;
        let (b_field, b_is_constant) = modulo.left.as_index_and_constant_flag();
        let (c_field, c_is_constant) = modulo.right.as_index_and_constant_flag();
        let b_type = modulo.left.as_type_code();
        let c_type = modulo.right.as_type_code();

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

impl Display for Modulo {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let Modulo {
            destination,
            left,
            right,
        } = self;

        match left.as_type_code() {
            TypeCode::BOOLEAN => write!(f, "R_BOOL_{destination}")?,
            TypeCode::BYTE => write!(f, "R_BYTE_{destination}")?,
            TypeCode::CHARACTER => write!(f, "R_STR_{destination}")?,
            TypeCode::FLOAT => write!(f, "R_FLOAT_{destination}")?,
            TypeCode::INTEGER => write!(f, "R_INT_{destination}")?,
            TypeCode::STRING => write!(f, "R_STR_{destination}")?,
            _ => todo!(),
        }

        write!(f, " = {left} % {right}",)
    }
}
