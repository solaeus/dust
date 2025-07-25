use crate::{Thread, vm::CallFrame};

#[unsafe(no_mangle)]
pub unsafe extern "C" fn read_line(_: &mut Thread, _: &mut CallFrame) {
    todo!()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn write_line(_: &mut Thread, _: &mut CallFrame) {
    todo!()
}
