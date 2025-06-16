use crate::{
    Address,
    panic_vm::{CallFrame, Memory, ThreadPool},
    r#type::TypeKind,
};

pub fn spawn<C>(
    _: Address,
    arguments: &[(Address, TypeKind)],
    call: &mut CallFrame<C>,
    memory: &mut Memory<C>,
    threads: &ThreadPool<C>,
) {
    todo!();
}
