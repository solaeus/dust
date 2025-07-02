use std::sync::{Arc, RwLock};

use crate::{
    Address, OperandType,
    panic_vm::{CallFrame, Cell, Memory, ThreadPool},
};

pub fn spawn<C>(
    _: Address,
    arguments: &[(Address, OperandType)],
    call: &mut CallFrame<C>,
    cells: &Arc<RwLock<Vec<Cell<C>>>>,
    threads: &ThreadPool<C>,
) {
    todo!();
}
