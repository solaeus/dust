#![macro_use]

use std::sync::{Arc, RwLock};

use crate::{
    Address, Chunk, DustString, MemoryKind, OperandType,
    panic_vm::{CallFrame, Cell, HeapSlot, Memory, ThreadPool, macros::*},
};

pub fn int_to_str<C: Chunk>(
    destination: Address,
    arguments: &[(Address, OperandType)],
    call: &mut CallFrame<C>,
    memory: &mut Memory<C>,
    cells: &Arc<RwLock<Vec<Cell<C>>>>,
    _: &ThreadPool<C>,
) {
    let integer = get_integer!(arguments[0].0, memory, call.chunk, cells);
    let string = DustString::from(integer.to_string());

    set_string!(destination, memory, cells, string);
}
