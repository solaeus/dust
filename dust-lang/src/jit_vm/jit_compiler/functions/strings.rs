use std::{ptr, slice};

use crate::{
    Object,
    jit_vm::{jit_compiler::ERROR_REPLACEMENT_STR, thread::ThreadContext},
};

pub unsafe extern "C" fn allocate_string(
    string_pointer: *mut u8,
    string_length: usize,
    thread_context: *mut ThreadContext,
) -> i64 {
    let thread_context = unsafe { &mut *thread_context };

    let object_pool = unsafe { &mut *thread_context.object_pool_pointer };
    let register_stack = unsafe { &mut *thread_context.register_stack_vec_pointer };
    let register_tags = unsafe { &mut *thread_context.register_tags_vec_pointer };
    let register_stack_used_length = unsafe { *thread_context.register_stack_used_length_pointer };
    let register_window = &register_stack[0..register_stack_used_length];
    let register_tags_window = &register_tags[0..register_stack_used_length];

    let bytes = unsafe { slice::from_raw_parts(string_pointer, string_length).to_vec() };
    let string = unsafe { String::from_utf8_unchecked(bytes) };

    let object = Object::string(string);
    let object_pointer = object_pool.allocate(object, register_window, register_tags_window);

    object_pointer as i64
}

pub unsafe extern "C" fn concatenate_strings(
    left: *const Object,
    right: *const Object,
    thread_context: *const ThreadContext,
) -> i64 {
    let thread_context = unsafe { &*thread_context };
    let object_pool = unsafe { &mut *thread_context.object_pool_pointer };
    let register_stack = unsafe { &mut *thread_context.register_stack_vec_pointer };
    let register_tags = unsafe { &mut *thread_context.register_tags_vec_pointer };
    let register_stack_used_length = unsafe { *thread_context.register_stack_used_length_pointer };
    let register_window = &register_stack[0..register_stack_used_length];
    let register_tags_window = &register_tags[0..register_stack_used_length];

    let concatenated = if ptr::eq(left, right) {
        let right_string = unsafe { &*right }
            .as_string()
            .cloned()
            .unwrap_or_else(|| ERROR_REPLACEMENT_STR.to_string());
        let left_string = unsafe { &*left }
            .as_string()
            .map(|string| string.as_str())
            .unwrap_or(ERROR_REPLACEMENT_STR);
        let mut concatenated = String::with_capacity(left_string.len() + right_string.len());

        concatenated.push_str(left_string);
        concatenated.push_str(&right_string);

        concatenated
    } else {
        let left_string = unsafe { &*left }
            .as_string()
            .map(|string| string.as_str())
            .unwrap_or(ERROR_REPLACEMENT_STR);
        let right_string = unsafe { &*right }
            .as_string()
            .map(|string| string.as_str())
            .unwrap_or(ERROR_REPLACEMENT_STR);
        let mut concatenated = String::with_capacity(left_string.len() + right_string.len());

        concatenated.push_str(left_string);
        concatenated.push_str(right_string);

        concatenated
    };
    let object = Object::string(concatenated);
    let object_pointer = object_pool.allocate(object, register_window, register_tags_window);

    object_pointer as i64
}
