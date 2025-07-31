use crate::{Object, Thread};

/// # Safety
/// All arguments are raw pointers and must be valid.
pub unsafe extern "C" fn concatenate_strings(
    thread: *mut Thread,
    left_pointer: i64,
    right_pointer: i64,
) -> i64 {
    let thread = unsafe { &mut *thread };
    let left_pointer = left_pointer as *const Object;
    let left_object = unsafe { &*left_pointer };
    let left_string = left_object.as_string().cloned().unwrap_or_default();
    let right_pointer = right_pointer as *const Object;
    let right_object = unsafe { &*right_pointer };
    let right_string = right_object.as_string().cloned().unwrap_or_default();
    let concatenated = format!("{left_string}{right_string}");

    thread.object_pool.allocate(Object::string(concatenated)) as i64
}
