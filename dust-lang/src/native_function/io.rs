use std::io::{Write, stdin, stdout};

use crate::{
    Address,
    panic_vm::{CallFrame, Memory, ThreadPool},
    r#type::TypeKind,
};

pub fn read_line<C>(
    destination: Address,
    _: &[(Address, TypeKind)],
    _: &mut CallFrame<C>,
    memory: &mut Memory<C>,
    _: &ThreadPool<C>,
) {
    todo!()
}

pub fn write_line<C>(
    _: Address,
    arguments: &[(Address, TypeKind)],
    _: &mut CallFrame<C>,
    memory: &mut Memory<C>,
    _: &ThreadPool<C>,
) {
    todo!()
}
