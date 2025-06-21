use std::sync::{Arc, RwLock};

use crate::{
    Address, Chunk, OperandType,
    panic_vm::{CallFrame, Cell, Memory, ThreadPool, macros::*},
};

pub fn read_line<C>(
    destination: Address,
    _: &[(Address, OperandType)],
    _: &mut CallFrame<C>,
    memory: &mut Memory<C>,
    cells: &Arc<RwLock<Vec<Cell<C>>>>,
    _: &ThreadPool<C>,
) {
    todo!()
}

pub fn write_line<C: Chunk>(
    _: Address,
    arguments: &[(Address, OperandType)],
    call: &mut CallFrame<C>,
    memory: &mut Memory<C>,
    cells: &Arc<RwLock<Vec<Cell<C>>>>,
    _: &ThreadPool<C>,
) {
    todo!()
}
