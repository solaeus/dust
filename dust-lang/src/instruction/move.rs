use std::fmt::{self, Display, Formatter};

use crate::{Instruction, Operation};

use super::{Address, Destination, InstructionFields};

pub struct Move {
    pub destination: Destination,
    pub operand: Address,
}

impl From<&Instruction> for Move {
    fn from(instruction: &Instruction) -> Self {
        Move {
            destination: instruction.destination(),
            operand: instruction.b_address(),
        }
    }
}

impl From<Move> for Instruction {
    fn from(r#move: Move) -> Self {
        let operation = Operation::MOVE;
        let Destination {
            index: a_field,
            is_register: a_is_register,
        } = r#move.destination;
        let Address {
            index: b_field,
            kind: b_kind,
        } = r#move.operand;

        InstructionFields {
            operation,
            a_field,
            a_is_register,
            b_field,
            b_kind,
            ..Default::default()
        }
        .build()
    }
}

impl Display for Move {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let Move {
            destination,
            operand,
        } = self;
        let destination_address = destination.as_address(operand.r#type());

        write!(f, "{destination_address} = {operand}")
    }
}
