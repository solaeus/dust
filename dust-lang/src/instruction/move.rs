use crate::{Instruction, Operation};

pub struct Move {
    pub from: u8,
    pub to: u8,
}

impl From<&Instruction> for Move {
    fn from(instruction: &Instruction) -> Self {
        Move {
            from: instruction.b_field(),
            to: instruction.c_field(),
        }
    }
}

impl From<Move> for Instruction {
    fn from(r#move: Move) -> Self {
        let operation = Operation::MOVE;
        let b = r#move.from;
        let c = r#move.to;

        Instruction::new(operation, 0, b, c, false, false, false)
    }
}
