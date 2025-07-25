#![macro_use]

use crate::vm::CallFrame;

#[unsafe(no_mangle)]
pub extern "C" fn int_to_str(_: &mut crate::vm::Thread, _: &mut CallFrame) {
    todo!()
}
