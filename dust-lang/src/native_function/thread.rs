use std::sync::Arc;

use crate::{
    Address, Destination,
    instruction::AddressKind,
    panic_vm::{CallFrame, Memory, Thread, ThreadPool, macros::*},
};

pub fn spawn<const REGISTER_COUNT: usize>(
    _: Destination,
    arguments: &[Address],
    call: &mut CallFrame,
    memory: &mut Memory<REGISTER_COUNT>,
    threads: &ThreadPool<REGISTER_COUNT>,
) {
    let function_address = arguments[0];
    let function = match function_address.kind {
        AddressKind::FUNCTION_REGISTER => {
            get_register!(memory, functions, function_address)
        }
        AddressKind::FUNCTION_MEMORY => {
            get_memory!(memory, functions, function_address)
        }
        AddressKind::FUNCTION_PROTOTYPE => {
            get_constant!(call.chunk, prototypes, function_address)
        }
        AddressKind::FUNCTION_SELF => &call.chunk,
        _ => unreachable!(),
    };
    let function = Arc::clone(function);
    let thread = Thread::<REGISTER_COUNT>::new(function, Arc::clone(threads));

    threads.lock().expect("Failed to lock threads").push(thread);
}
