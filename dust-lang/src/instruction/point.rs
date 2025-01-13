use std::fmt::{self, Display, Formatter};

use crate::{Instruction, Operation};

use super::InstructionBuilder;

pub struct Point {
    pub from: u16,
    pub to: u16,
}

impl From<Instruction> for Point {
    fn from(instruction: Instruction) -> Self {
        Point {
            from: instruction.b_field(),
            to: instruction.c_field(),
        }
    }
}

impl From<Point> for Instruction {
    fn from(r#move: Point) -> Self {
        let operation = Operation::POINT;
        let b_field = r#move.from;
        let c_field = r#move.to;

        InstructionBuilder {
            operation,
            b_field,
            c_field,
            ..Default::default()
        }
        .build()
    }
}

impl Display for Point {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let Point { from, to } = self;

        write!(f, "{from} -> {to}")
    }
}
