use std::io::{Write, stdin, stdout};

use crate::{
    Address, DustString,
    instruction::MemoryKind,
    panic_vm::{CallFrame, Memory, ThreadPool},
    r#type::TypeKind,
};

pub fn read_line(
    destination: Address,
    _: &[(Address, TypeKind)],
    _: &mut CallFrame,
    memory: &mut Memory,
    _: &ThreadPool,
) {
    todo!()
}

pub fn write_line(
    _: Address,
    arguments: &[(Address, TypeKind)],
    _: &mut CallFrame,
    memory: &mut Memory,
    _: &ThreadPool,
) {
    todo!()
}
