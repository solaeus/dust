use std::slice;

use crate::{
    Object, Register, Thread,
    jit_vm::{ObjectPool, RegisterTag},
};

pub unsafe extern "C" fn allocate_string(
    string_pointer: *const u8,
    length: usize,
    object_pool_pointer: *mut ObjectPool,
    registers: *const Register,
    registers_length: usize,
    register_tags: *const RegisterTag,
) -> i64 {
    let borrowed_slice = unsafe { slice::from_raw_parts(string_pointer, length) };
    let string = unsafe { String::from_utf8_unchecked(borrowed_slice.to_vec()) };
    let object = Object::string(string);
    let object_pool = unsafe { &mut *object_pool_pointer };
    let registers = unsafe { slice::from_raw_parts(registers, registers_length) };
    let register_tags = unsafe { slice::from_raw_parts(register_tags, registers_length) };
    let object_pointer = object_pool.allocate(object, registers, register_tags);

    object_pointer as i64
}

/// # Safety
/// All arguments are raw pointers and must be valid.
pub unsafe extern "C" fn concatenate_strings(
    _thread: *mut Thread,
    _left_pointer: i64,
    _right_pointer: i64,
) -> i64 {
    // let thread = unsafe { &mut *thread };
    // let left_pointer = left_pointer as *const Object;
    // let left_object = unsafe { &*left_pointer };
    // let left_string = left_object.as_string().cloned().unwrap_or_default();
    // let right_pointer = right_pointer as *const Object;
    // let right_object = unsafe { &*right_pointer };
    // let right_string = right_object.as_string().cloned().unwrap_or_default();
    // let concatenated = format!("{left_string}{right_string}");

    todo!()

    // thread.object_pool.allocate(Object::string(concatenated)) as i64
}
