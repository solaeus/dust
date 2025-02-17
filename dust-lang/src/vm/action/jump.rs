use tracing::trace;

use crate::{vm::Thread, Instruction};

use super::Cache;

pub fn jump(ip: &mut usize, instruction: &Instruction, _: &mut Thread, _: &mut Cache) {
    let offset = instruction.b_field() as usize;
    let is_positive = instruction.c_field() != 0;

    if is_positive {
        trace!("JUMP +{}", offset);
    } else {
        trace!("JUMP -{}", offset);
    }

    if is_positive {
        *ip += offset;
    } else {
        *ip -= offset + 1;
    }
}

pub fn optimized_jump_forward(
    ip: &mut usize,
    instruction: &Instruction,
    _: &mut Thread,
    _: &mut Cache,
) {
    let offset = instruction.b_field() as usize;

    trace!("JUMP +{}", offset);

    *ip += offset;
}

pub fn optimized_jump_backward(
    ip: &mut usize,
    instruction: &Instruction,
    _: &mut Thread,
    _: &mut Cache,
) {
    let offset = instruction.b_field() as usize;

    trace!("JUMP -{}", offset);

    *ip -= offset + 1;
}
