use crate::{
    Address, OperandType,
    panic_vm::{CallFrame, Memory, ThreadPool},
};

pub fn spawn<C>(
    _: Address,
    arguments: &[(Address, OperandType)],
    call: &mut CallFrame<C>,
    memory: &mut Memory<C>,
    threads: &ThreadPool<C>,
) {
    todo!();
}
