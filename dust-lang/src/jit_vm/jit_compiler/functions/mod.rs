pub mod lists;
pub mod strings;

pub use lists::*;
pub use strings::*;
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

#[unsafe(no_mangle)]
pub extern "C" fn log_value(value: i64) {
    trace!("Value: {}", value);
}

#[unsafe(no_mangle)]
pub extern "C" fn log_call_frame(
    ip: i64,
    function_index: i64,
    register_range_start: i64,
    register_range_end: i64,
    arguments_index: i64,
    return_register_index: i64,
) {
    trace!(
        "Call frame: ip: {}, function_index: {}, register_range: {}-{}, arguments_index: {}, return_register_index: {}",
        ip,
        function_index,
        register_range_start,
        register_range_end,
        arguments_index,
        return_register_index
    );
}
