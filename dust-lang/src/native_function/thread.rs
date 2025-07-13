use std::sync::{Arc, RwLock};

use crate::{
    Address, OperandType,
    vm::{CallFrame, Cell, ThreadPool},
};

pub fn spawn(
    _: Address,
    arguments: &[(Address, OperandType)],
    call: &mut CallFrame,
    cells: &Arc<RwLock<Vec<Cell>>>,
    threads: &ThreadPool,
) {
    todo!();
}
