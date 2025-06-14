use crate::{
    Address,
    panic_vm::{CallFrame, Memory, ThreadPool},
    r#type::TypeKind,
};

pub fn spawn(
    _: Address,
    arguments: &[(Address, TypeKind)],
    call: &mut CallFrame,
    memory: &mut Memory,
    threads: &ThreadPool,
) {
    todo!();
}
