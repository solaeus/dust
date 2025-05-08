use crate::{
    Instruction,
    instruction::{AddressKind, Move},
};

use super::{Memory, Thread};

pub type Runner = fn(Instruction, Thread) -> Thread;

pub const RUNNERS: [Runner; 1] = [run_move];

fn run_move(instruction: Instruction, thread: Thread) -> Thread {
    let Move {
        destination,
        operand,
    } = Move::from(&instruction);

    let value_to_move = match operand.kind {
        AddressKind::BOOLEAN_MEMORY => thread
            .current_memory
            .heap_slot_table
            .booleans
            .get(operand.index as usize),
        AddressKind::BOOLEAN_REGISTER => thread.current_memory.resolve_boolean_memory(address),
    };
}
