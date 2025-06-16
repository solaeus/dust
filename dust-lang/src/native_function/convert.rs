use crate::{
    Address, OperandType,
    panic_vm::{CallFrame, Memory, ThreadPool},
};

pub fn int_to_string<C>(
    destination: Address,
    arguments: &[(Address, OperandType)],
    call: &mut CallFrame<C>,
    memory: &mut Memory<C>,
    _: &ThreadPool<C>,
) {
    todo!()
}
