use std::fmt::{self, Display, Formatter};

use crate::{Instruction, Operation};

use super::{InstructionFields, Operand, TypeCode};

pub struct Point {
    pub destination: u16,
    pub to: Operand,
    pub r#type: TypeCode,
}

impl From<Instruction> for Point {
    fn from(instruction: Instruction) -> Self {
        Point {
            destination: instruction.a_field(),
            to: instruction.b_as_operand(),
            r#type: instruction.b_type(),
        }
    }
}

impl From<Point> for Instruction {
    fn from(r#move: Point) -> Self {
        let operation = Operation::POINT;
        let a_field = r#move.destination;
        let (b_field, b_is_constant) = r#move.to.as_index_and_constant_flag();
        let b_type = r#move.r#type;

        InstructionFields {
            operation,
            a_field,
            b_field,
            b_type,
            b_is_constant,
            ..Default::default()
        }
        .build()
    }
}

impl Display for Point {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let Point {
            destination,
            to,
            r#type,
        } = self;

        match *r#type {
            TypeCode::BOOLEAN => write!(f, "R_BOOL_{destination} -> R_BOOL_{to}"),
            TypeCode::BYTE => write!(f, "R_BYTE_{destination} -> R_BYTE_{to}"),
            TypeCode::CHARACTER => write!(f, "R_CHAR_{destination} -> R_CHAR_{to}"),
            TypeCode::FLOAT => write!(f, "R_FLOAT_{destination} -> R_FLOAT_{to}"),
            TypeCode::INTEGER => write!(f, "R_INT_{destination} -> R_INT_{to}"),
            TypeCode::STRING => write!(f, "R_STR_{destination} -> R_STR_{to}"),
            unsupported => write!(
                f,
                "Unsupported type code: {unsupported} for Point instruction"
            ),
        }
    }
}
