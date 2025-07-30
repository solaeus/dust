use crate::{Object, Thread, Value};

/// # Safety
/// This function dereferences a raw pointer and must only be called with a valid ThreadRunner pointer.
pub unsafe extern "C" fn set_return_value_to_integer(thread: *mut Thread, integer_value: i64) {
    unsafe {
        (*thread).return_value = Some(Value::Integer(integer_value));
    }
}

/// # Safety
/// This function dereferences a raw pointer and must only be called with a valid ThreadRunner pointer.
pub unsafe extern "C" fn set_return_value_to_string(
    thread: *mut Thread,
    object_ptr: i64, // Actually a pointer, cast as i64
) {
    let object_ptr = object_ptr as *const Object;
    let string = if !object_ptr.is_null() {
        unsafe { (*object_ptr).as_string().cloned().unwrap_or_default() }
    } else {
        String::new()
    };

    unsafe {
        (*thread).return_value = Some(Value::String(string));
    }
}
