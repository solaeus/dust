use std::{ptr, slice};

use crate::{Object, jit_vm::thread::ThreadContext};

pub unsafe extern "C" fn allocate_string(
    string_pointer: *mut u8,
    string_length: usize,
    thread_context: *mut ThreadContext,
    register_range_start: usize,
    register_range_end: usize,
) -> i64 {
    let thread_context = unsafe { &mut *thread_context };

    let object_pool = unsafe { &mut *thread_context.object_pool_pointer };
    let registers = unsafe { &mut *thread_context.register_stack_vec_pointer };
    let register_tags = unsafe { &mut *thread_context.register_tags_vec_pointer };

    let register_window = &mut registers[register_range_start..register_range_end];
    let register_tags_window = &mut register_tags[register_range_start..register_range_end];

    let bytes = unsafe { slice::from_raw_parts_mut(string_pointer, string_length) };
    let string = unsafe { String::from_utf8_unchecked(bytes.to_vec()) };

    let object = Object::string(string);
    let object_pointer = object_pool.allocate(object, register_window, register_tags_window);

    object_pointer as i64
}

pub unsafe extern "C" fn concatenate_strings(
    left: *const Object,
    right: *const Object,
    thread_context: *const ThreadContext,
    register_range_start: usize,
    register_range_end: usize,
) -> i64 {
    let thread_context = unsafe { &*thread_context };
    let object_pool = unsafe { &mut *thread_context.object_pool_pointer };
    let register_stack = unsafe { &mut *thread_context.register_stack_vec_pointer };
    let register_tags = unsafe { &mut *thread_context.register_tags_vec_pointer };
    let register_window = &mut register_stack[register_range_start..register_range_end];
    let register_tags_window = &mut register_tags[register_range_start..register_range_end];

    let concatenated = if ptr::eq(left, right) {
        let right_string = unsafe { &*right }
            .as_string()
            .expect("Expected a string object")
            .clone();
        let left_string = unsafe { &*left }
            .as_string()
            .expect("Expected a string object");
        let mut concatenated = String::with_capacity(left_string.len() + right_string.len());

        concatenated.push_str(left_string);
        concatenated.push_str(&right_string);

        concatenated
    } else {
        let left_string = unsafe { &*left }
            .as_string()
            .expect("Expected a string object");
        let right_string = unsafe { &*right }
            .as_string()
            .expect("Expected a string object");
        let mut concatenated = String::with_capacity(left_string.len() + right_string.len());

        concatenated.push_str(left_string);
        concatenated.push_str(right_string);

        concatenated
    };
    let object = Object::string(concatenated);
    let object_pointer = object_pool.allocate(object, register_window, register_tags_window);

    object_pointer as i64
}
