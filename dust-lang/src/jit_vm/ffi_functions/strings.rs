use std::{ptr, slice};

use crate::jit_vm::{ERROR_REPLACEMENT_STR, Object, thread_pool::ThreadContext};

#[unsafe(no_mangle)]
pub unsafe extern "C" fn allocate_string(
    string_pointer: *mut u8,
    string_length: usize,
    thread_context: *mut ThreadContext,
) -> i64 {
    let thread_context = unsafe { &mut *thread_context };

    let object_pool = unsafe { &mut *thread_context.object_pool_pointer };
    let register_stack = unsafe { &mut *thread_context.register_vec_pointer };
    let register_tags = unsafe { &mut *thread_context.register_tag_vec_pointer };
    let register_window = &register_stack[0..thread_context.registers_used];
    let register_tags_window = &register_tags[0..thread_context.registers_used];

    let bytes = unsafe { slice::from_raw_parts(string_pointer, string_length).to_vec() };
    let string = unsafe { String::from_utf8_unchecked(bytes) };

    let object = Object::string(string);
    let object_pointer = object_pool.allocate(object, register_window, register_tags_window);

    object_pointer as i64
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn concatenate_strings(
    left: *const Object,
    right: *const Object,
    thread_context: *const ThreadContext,
) -> i64 {
    let thread_context = unsafe { &*thread_context };
    let object_pool = unsafe { &mut *thread_context.object_pool_pointer };
    let register_stack = unsafe { &mut *thread_context.register_vec_pointer };
    let register_tags = unsafe { &mut *thread_context.register_tag_vec_pointer };
    let register_window = &register_stack[0..thread_context.registers_used];
    let register_tags_window = &register_tags[0..thread_context.registers_used];

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

#[unsafe(no_mangle)]
pub unsafe extern "C" fn concatenate_character_string(
    left: i64,
    right: *const Object,
    thread_context: *const ThreadContext,
) -> i64 {
    let thread_context = unsafe { &*thread_context };
    let object_pool = unsafe { &mut *thread_context.object_pool_pointer };
    let register_stack = unsafe { &mut *thread_context.register_vec_pointer };
    let register_tags = unsafe { &mut *thread_context.register_tag_vec_pointer };
    let register_window = &register_stack[0..thread_context.registers_used];
    let register_tags_window = &register_tags[0..thread_context.registers_used];

    let left_char = std::char::from_u32(left as u32).unwrap_or_default();
    let right_string = unsafe { &*right }
        .as_string()
        .map(|string| string.as_str())
        .unwrap_or(ERROR_REPLACEMENT_STR);
    let mut concatenated = String::with_capacity(left_char.len_utf8() + right_string.len());

    concatenated.push(left_char);
    concatenated.push_str(right_string);

    let object = Object::string(concatenated);
    let object_pointer = object_pool.allocate(object, register_window, register_tags_window);

    object_pointer as i64
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn concatenate_string_character(
    left: *const Object,
    right: i64,
    thread_context: *const ThreadContext,
) -> i64 {
    let thread_context = unsafe { &*thread_context };
    let object_pool = unsafe { &mut *thread_context.object_pool_pointer };
    let register_stack = unsafe { &mut *thread_context.register_vec_pointer };
    let register_tags = unsafe { &mut *thread_context.register_tag_vec_pointer };
    let register_window = &register_stack[0..thread_context.registers_used];
    let register_tags_window = &register_tags[0..thread_context.registers_used];

    let left_string = unsafe { &*left }
        .as_string()
        .map(|string| string.as_str())
        .unwrap_or(ERROR_REPLACEMENT_STR);
    let right_char = std::char::from_u32(right as u32).unwrap_or_default();
    let mut concatenated = String::with_capacity(left_string.len() + right_char.len_utf8());

    concatenated.push_str(left_string);
    concatenated.push(right_char);

    let object = Object::string(concatenated);
    let object_pointer = object_pool.allocate(object, register_window, register_tags_window);

    object_pointer as i64
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn concatenate_characters(
    left: i64,
    right: i64,
    thread_context: *const ThreadContext,
) -> i64 {
    let thread_context = unsafe { &*thread_context };
    let object_pool = unsafe { &mut *thread_context.object_pool_pointer };
    let register_stack = unsafe { &mut *thread_context.register_vec_pointer };
    let register_tags = unsafe { &mut *thread_context.register_tag_vec_pointer };
    let register_window = &register_stack[0..thread_context.registers_used];
    let register_tags_window = &register_tags[0..thread_context.registers_used];

    let left_char = std::char::from_u32(left as u32).unwrap_or_default();
    let right_char = std::char::from_u32(right as u32).unwrap_or_default();
    let mut concatenated = String::with_capacity(left_char.len_utf8() + right_char.len_utf8());

    concatenated.push(left_char);
    concatenated.push(right_char);

    let object = Object::string(concatenated);
    let object_pointer = object_pool.allocate(object, register_window, register_tags_window);

    object_pointer as i64
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn compare_strings_equal(left: *const Object, right: *const Object) -> i8 {
    let left_string = unsafe { &*left }
        .as_string()
        .map(|string| string.as_str())
        .unwrap_or(ERROR_REPLACEMENT_STR);
    let right_string = unsafe { &*right }
        .as_string()
        .map(|string| string.as_str())
        .unwrap_or(ERROR_REPLACEMENT_STR);

    (left_string == right_string) as i8
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn compare_strings_less_than(
    left: *const Object,
    right: *const Object,
) -> i8 {
    let left_string = unsafe { &*left }
        .as_string()
        .map(|string| string.as_str())
        .unwrap_or(ERROR_REPLACEMENT_STR);
    let right_string = unsafe { &*right }
        .as_string()
        .map(|string| string.as_str())
        .unwrap_or(ERROR_REPLACEMENT_STR);

    (left_string < right_string) as i8
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn compare_strings_less_than_equal(
    left: *const Object,
    right: *const Object,
) -> i8 {
    let left_string = unsafe { &*left }
        .as_string()
        .map(|string| string.as_str())
        .unwrap_or(ERROR_REPLACEMENT_STR);
    let right_string = unsafe { &*right }
        .as_string()
        .map(|string| string.as_str())
        .unwrap_or(ERROR_REPLACEMENT_STR);

    (left_string <= right_string) as i8
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn integer_to_string(value: i64, thread_context: *mut ThreadContext) -> i64 {
    let thread_context = unsafe { &*thread_context };
    let object_pool = unsafe { &mut *thread_context.object_pool_pointer };
    let register_stack = unsafe { &mut *thread_context.register_vec_pointer };
    let register_tags = unsafe { &mut *thread_context.register_tag_vec_pointer };
    let register_window = &register_stack[0..thread_context.registers_used];
    let register_tags_window = &register_tags[0..thread_context.registers_used];

    let string = value.to_string();
    let object = Object::string(string);
    let object_pointer = object_pool.allocate(object, register_window, register_tags_window);

    object_pointer as i64
}
