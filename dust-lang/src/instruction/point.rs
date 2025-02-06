use std::fmt::{self, Display, Formatter};

use crate::{Instruction, Operation};

use super::{InstructionFields, Operand};

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

        InstructionFields {
            operation,
            a_field,
            b_field,
            b_is_constant,
            ..Default::default()
        }
        .build()
    }
}

impl Display for Point {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let Point { destination, to } = self;

        write!(f, "R{destination} -> {to}")
    }
}
