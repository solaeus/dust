use crate::{Thread, vm::CallFrame};

#[unsafe(no_mangle)]
pub extern "C" fn spawn(_: &mut Thread, _: &mut CallFrame) {
    todo!();
}
