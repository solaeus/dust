use crate::{ThreadRunner, Value};

/// # Safety
/// This function dereferences a raw pointer and must only be called with a valid ThreadRunner pointer.
pub unsafe extern "C" fn set_return_value_to_integer(
    thread_runner: *mut ThreadRunner,
    integer_value: i64,
) {
    unsafe {
        (*thread_runner).return_value = Some(Value::Integer(integer_value));
    }
}
