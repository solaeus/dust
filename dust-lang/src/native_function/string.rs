use crate::{
    Address,
    panic_vm::{CallFrame, Memory, ThreadPool},
    r#type::TypeKind,
};

pub fn to_string(
    destination: Address,
    arguments: &[(Address, TypeKind)],
    call: &mut CallFrame,
    memory: &mut Memory,
    _: &ThreadPool,
) {
    todo!()
}
