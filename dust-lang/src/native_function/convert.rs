#![macro_use]

use std::sync::{Arc, RwLock};

use crate::{
    Address, Chunk, OperandType,
    vm::{CallFrame, Cell, ThreadPool},
};

pub fn int_to_str(
    destination: Address,
    arguments: &[(Address, OperandType)],
    call: &mut CallFrame,
    cells: &Arc<RwLock<Vec<Cell>>>,
    _: &ThreadPool,
) {
    todo!()
}
