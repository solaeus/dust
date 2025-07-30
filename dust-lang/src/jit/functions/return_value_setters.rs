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

/// # Safety
/// This function dereferences a raw pointer and must only be called with a valid ThreadRunner pointer.
pub unsafe extern "C" fn set_return_value_to_string(
    thread_runner: *mut ThreadRunner,
    object_index: i64,
) {
    let thread_runner = unsafe { &mut *thread_runner };
    let string = thread_runner
        .object_pool
        .get(object_index as usize)
        .and_then(|object| object.as_string())
        .cloned()
        .unwrap_or_default();

    thread_runner.return_value = Some(Value::String(string));
}
