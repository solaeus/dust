use crate::{
    Address,
    panic_vm::{CallFrame, Memory, ThreadPool},
    r#type::TypeKind,
};

pub fn to_string<C>(
    destination: Address,
    arguments: &[(Address, TypeKind)],
    call: &mut CallFrame<C>,
    memory: &mut Memory<C>,
    _: &ThreadPool<C>,
) {
    todo!()
}
