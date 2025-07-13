use std::sync::{Arc, RwLock};

use crate::{
    Address, Chunk, OperandType,
    vm::{CallFrame, Cell, ThreadPool},
};

pub fn read_line(
    destination: Address,
    _: &[(Address, OperandType)],
    _: &mut CallFrame,
    cells: &Arc<RwLock<Vec<Cell>>>,
    _: &ThreadPool,
) {
    todo!()
}

pub fn write_line(
    _: Address,
    arguments: &[(Address, OperandType)],
    call: &mut CallFrame,
    cells: &Arc<RwLock<Vec<Cell>>>,
    _: &ThreadPool,
) {
    todo!()
}
