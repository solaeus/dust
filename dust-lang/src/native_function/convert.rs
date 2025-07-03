#![macro_use]

use std::sync::{Arc, RwLock};

use crate::{
    Address, Chunk, DustString, MemoryKind, OperandType,
    vm::{CallFrame, Cell, Memory, ThreadPool, macros::*},
};

pub fn int_to_str<C: Chunk>(
    destination: Address,
    arguments: &[(Address, OperandType)],
    call: &mut CallFrame<C>,
    cells: &Arc<RwLock<Vec<Cell<C>>>>,
    _: &ThreadPool<C>,
) {
    todo!()
}
