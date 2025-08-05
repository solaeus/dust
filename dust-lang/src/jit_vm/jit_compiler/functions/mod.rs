pub mod string_manipulation;

pub use string_manipulation::*;
use tracing::trace;

use crate::Operation;

#[unsafe(no_mangle)]
pub extern "C" fn debug_check_pointer(ptr: *const u8) {
    if ptr.is_null() {
        panic!("Pointer is null!");
    }

    if !ptr.is_aligned() {
        panic!("Pointer is not aligned!");
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn log_operation(op_code: i64) {
    trace!("Running operation: {}", Operation(op_code as u8));
}
