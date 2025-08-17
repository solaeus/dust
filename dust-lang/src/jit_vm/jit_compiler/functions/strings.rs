use std::slice;

use crate::{
    Object, Register,
    jit_vm::{ObjectPool, RegisterTag, thread::ThreadContext},
};

pub unsafe extern "C" fn allocate_string(
    string_pointer: *const u8,
    string_length: usize,
    thread_context: *mut ThreadContext,
    register_range_start: usize,
    register_range_end: usize,
) -> i64 {
    let borrowed_slice = unsafe { slice::from_raw_parts(string_pointer, string_length) };
    let string = unsafe { String::from_utf8_unchecked(borrowed_slice.to_vec()) };
    let object = Object::string(string);
    let thread_context = unsafe { &mut *thread_context };
    let object_pool = unsafe { &mut *thread_context.object_pool_pointer };
    let registers = unsafe { &*thread_context.register_stack_vec_pointer };
    let register_tags = unsafe { &*thread_context.register_tags_vec_pointer };
    let register_window = &registers[register_range_start..register_range_end];
    let register_tags_window = &register_tags[register_range_start..register_range_end];
    let object_pointer = object_pool.allocate(object, register_window, register_tags_window);

    object_pointer as i64
}

pub unsafe extern "C" fn _concatenate_strings(
    left_pointer: *const u8,
    left_length: usize,
    right_pointer: *const u8,
    right_length: usize,
    thread_context: *mut ThreadContext,
    register_range_start: usize,
    register_range_end: usize,
) -> i64 {
    let left_slice = unsafe { slice::from_raw_parts(left_pointer, left_length) };
    let right_slice = unsafe { slice::from_raw_parts(right_pointer, right_length) };
    let concatenated = unsafe {
        let mut vec = Vec::with_capacity(left_length + right_length);

        vec.extend_from_slice(left_slice);
        vec.extend_from_slice(right_slice);

        String::from_utf8_unchecked(vec)
    };
    let thread_context = unsafe { &mut *thread_context };
    let object_pool = unsafe { &mut *thread_context.object_pool_pointer };
    let registers = unsafe { &*thread_context.register_stack_vec_pointer };
    let register_tags = unsafe { &*thread_context.register_tags_vec_pointer };
    let register_window = &registers[register_range_start..register_range_end];
    let register_tags_window = &register_tags[register_range_start..register_range_end];
    let object = Object::string(concatenated);
    let object_pointer = object_pool.allocate(object, register_window, register_tags_window);

    object_pointer as i64
}
