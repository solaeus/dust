use std::fmt::{self, Display, Formatter};

use crate::{Instruction, Operation};

use super::{InstructionFields, Operand, TypeCode};

pub struct Point {
    pub destination: u16,
    pub to: Operand,
}

impl From<Instruction> for Point {
    fn from(instruction: Instruction) -> Self {
        Point {
            destination: instruction.a_field(),
            to: instruction.b_as_operand(),
        }
    }
}

impl From<Point> for Instruction {
    fn from(r#move: Point) -> Self {
        let operation = Operation::POINT;
        let a_field = r#move.destination;
        let (b_field, b_is_constant) = r#move.to.as_index_and_constant_flag();
        let b_type = r#move.to.as_type();

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
        let Point { destination, to } = self;

        match to.as_type() {
            TypeCode::BOOLEAN => write!(f, "R_BOOL_{destination} -> {to}"),
            TypeCode::BYTE => write!(f, "R_BYTE_{destination} -> {to}"),
            TypeCode::CHARACTER => write!(f, "R_CHAR_{destination} -> {to}"),
            TypeCode::FLOAT => write!(f, "R_FLOAT_{destination} -> {to}"),
            TypeCode::INTEGER => write!(f, "R_INT_{destination} -> {to}"),
            TypeCode::STRING => write!(f, "R_STR_{destination} -> {to}"),
            unsupported => write!(
                f,
                "Unsupported type code: {unsupported} for Point instruction"
            ),
        }
    }
}
