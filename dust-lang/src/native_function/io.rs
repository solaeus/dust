use crate::{
    Address, OperandType,
    panic_vm::{CallFrame, Memory, ThreadPool},
};

pub fn read_line<C>(
    destination: Address,
    _: &[(Address, OperandType)],
    _: &mut CallFrame<C>,
    memory: &mut Memory<C>,
    _: &ThreadPool<C>,
) {
    todo!()
}

pub fn write_line<C>(
    _: Address,
    arguments: &[(Address, OperandType)],
    _: &mut CallFrame<C>,
    memory: &mut Memory<C>,
    _: &ThreadPool<C>,
) {
    todo!()
}
