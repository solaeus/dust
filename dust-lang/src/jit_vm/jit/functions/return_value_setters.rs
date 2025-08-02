use crate::{Object, Thread, Value};

/// # Safety
/// `thread` is a raw pointer and must be valid.
pub unsafe extern "C" fn set_thread_return_value_to_integer(
    thread: *mut Thread,
    integer_value: i64,
) {
    unsafe {
        (*thread).return_value = Some(Value::Integer(integer_value));
    }
}

/// # Safety
/// `thread` and `object_pointer` are raw pointers and must be valid.
pub unsafe extern "C" fn set_thread_return_value_to_string(
    thread: *mut Thread,
    object_pointer: i64,
) {
    let object_ptr = object_pointer as *const Object;
    let string = unsafe { (*object_ptr).as_string().cloned().unwrap_or_default() };

    unsafe {
        (*thread).return_value = Some(Value::String(string));
    }
}
