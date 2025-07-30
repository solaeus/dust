use crate::{Object, ThreadRunner};

/// # Safety
/// This function dereferences a raw pointer and must only be called with a valid ThreadRunner pointer.
pub unsafe extern "C" fn concatenate_strings(
    thread_runner: *mut ThreadRunner,
    left_pointer: i64,
    rigth_pointer: i64,
) -> i64 {
    let left_pointer = left_pointer as *const Object;
    let right_pointer = rigth_pointer as *const Object;
    let left_string = if !left_pointer.is_null() {
        unsafe { (*left_pointer).as_string().cloned().unwrap_or_default() }
    } else {
        String::new()
    };
    let right_string = if !right_pointer.is_null() {
        unsafe { (*right_pointer).as_string().cloned().unwrap_or_default() }
    } else {
        String::new()
    };
    let concatenated = format!("{left_string}{right_string}");
    let thread_runner = unsafe { &mut *thread_runner };

    thread_runner
        .object_pool
        .allocate(Object::string(concatenated)) as i64
}
