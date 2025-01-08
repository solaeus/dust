use std::fmt::{self, Display, Formatter};

use crate::{Instruction, Operation};

pub struct Point {
    pub from: u8,
    pub to: u8,
}

impl From<Instruction> for Point {
    fn from(instruction: Instruction) -> Self {
        Point {
            from: instruction.b_field(),
            to: instruction.c_field(),
        }
    }
}

impl Display for Point {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let Point { from, to } = self;

        write!(f, "{from} -> {to}")
    }
}

impl From<Point> for Instruction {
    fn from(r#move: Point) -> Self {
        let operation = Operation::POINT;
        let b = r#move.from;
        let c = r#move.to;

        Instruction::new(operation, 0, b, c, false, false, false)
    }
}
