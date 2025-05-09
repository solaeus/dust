use crate::{
    Instruction,
    instruction::{AddressKind, Move},
};

use super::Thread;

pub type Runner = fn(Instruction, Thread) -> Thread;

pub const RUNNERS: [Runner; 1] = [run_move];

fn run_move(instruction: Instruction, mut thread: Thread) -> Thread {
    let Move {
        destination: to,
        operand: from,
    } = Move::from(&instruction);

    match from.kind {
        AddressKind::BOOLEAN_MEMORY => {
            let boolean = *thread
                .current_memory
                .booleans
                .get(from.index as usize)
                .unwrap()
                .as_value();

            *thread
                .current_memory
                .booleans
                .get_mut(to.index as usize)
                .unwrap()
                .as_value_mut() = boolean;
        }
        AddressKind::BOOLEAN_REGISTER => {
            let boolean = *thread
                .current_memory
                .register_table
                .booleans
                .get(from.index);

            thread
                .current_memory
                .register_table
                .booleans
                .set(to.index, boolean);
        }
        _ => unimplemented!(),
    };

    thread
}
