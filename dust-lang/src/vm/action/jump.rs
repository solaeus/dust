use tracing::trace;

use crate::{
    instruction::InstructionFields,
    vm::{call_frame::PointerCache, Thread},
};

pub fn jump(ip: &mut usize, instruction: &InstructionFields, _: &mut Thread) {
    let offset = instruction.b_field as usize;
    let is_positive = instruction.c_field != 0;

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
