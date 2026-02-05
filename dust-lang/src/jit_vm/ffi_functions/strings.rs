use std::{ptr, slice};

use crate::jit_vm::{Object, STRING_ERROR_TEXT, thread_pool::ThreadContext};

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

    object_pool.allocate(object, register_window, register_tags_window) as i64
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn concatenate_strings(
    left_pointer: *mut Object,
    right_pointer: *mut Object,
    thread_context: *mut ThreadContext,
) -> i64 {
    let thread_context = unsafe { &*thread_context };
    let object_pool = unsafe { &mut *thread_context.object_pool_pointer };
    let register_stack = unsafe { &mut *thread_context.register_vec_pointer };
    let register_tags = unsafe { &mut *thread_context.register_tag_vec_pointer };
    let register_window = &register_stack[0..thread_context.registers_used];
    let register_tags_window = &register_tags[0..thread_context.registers_used];

    let concatenated = if ptr::eq(left_pointer, right_pointer) {
        let right_string = unsafe { &*right_pointer }
            .as_string()
            .cloned()
            .expect(STRING_ERROR_TEXT);
        let left_string = unsafe { &*left_pointer }
            .as_string()
            .map(|string| string.as_str())
            .expect(STRING_ERROR_TEXT);
        let mut concatenated = String::with_capacity(left_string.len() + right_string.len());

        concatenated.push_str(left_string);
        concatenated.push_str(&right_string);

        concatenated
    } else {
        let left_string = unsafe { &*left_pointer }
            .as_string()
            .map(|string| string.as_str())
            .expect(STRING_ERROR_TEXT);
        let right_string = unsafe { &*right_pointer }
            .as_string()
            .map(|string| string.as_str())
            .expect(STRING_ERROR_TEXT);
        let mut concatenated = String::with_capacity(left_string.len() + right_string.len());

        concatenated.push_str(left_string);
        concatenated.push_str(right_string);

        concatenated
    };
    let object = Object::string(concatenated);

    object_pool.allocate(object, register_window, register_tags_window) as i64
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn concatenate_character_string(
    character: i64,
    object_pointer: *mut Object,
    thread_context: *mut ThreadContext,
) -> i64 {
    let thread_context = unsafe { &*thread_context };
    let object_pool = unsafe { &mut *thread_context.object_pool_pointer };
    let register_stack = unsafe { &mut *thread_context.register_vec_pointer };
    let register_tags = unsafe { &mut *thread_context.register_tag_vec_pointer };
    let register_window = &register_stack[0..thread_context.registers_used];
    let register_tags_window = &register_tags[0..thread_context.registers_used];

    let left_character = unsafe { char::from_u32_unchecked(character as u32) };
    let right_string = unsafe { &*object_pointer }
        .as_string()
        .map(|string| string.as_str())
        .expect(STRING_ERROR_TEXT);
    let mut concatenated = String::with_capacity(left_character.len_utf8() + right_string.len());

    concatenated.push(left_character);
    concatenated.push_str(right_string);

    let object = Object::string(concatenated);

    object_pool.allocate(object, register_window, register_tags_window) as i64
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn concatenate_string_character(
    object_pointer: *mut Object,
    character: i64,
    thread_context: *mut ThreadContext,
) -> i64 {
    let thread_context = unsafe { &*thread_context };
    let object_pool = unsafe { &mut *thread_context.object_pool_pointer };
    let register_stack = unsafe { &mut *thread_context.register_vec_pointer };
    let register_tags = unsafe { &mut *thread_context.register_tag_vec_pointer };
    let register_window = &register_stack[0..thread_context.registers_used];
    let register_tags_window = &register_tags[0..thread_context.registers_used];

    let left_string = unsafe { &*object_pointer }
        .as_string()
        .map(|string| string.as_str())
        .expect(STRING_ERROR_TEXT);
    let right_character = unsafe { char::from_u32_unchecked(character as u32) };
    let mut concatenated = String::with_capacity(left_string.len() + right_character.len_utf8());

    concatenated.push_str(left_string);
    concatenated.push(right_character);

    let object = Object::string(concatenated);

    object_pool.allocate(object, register_window, register_tags_window) as i64
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn concatenate_characters(
    left: i64,
    right: i64,
    thread_context: *mut ThreadContext,
) -> i64 {
    let thread_context = unsafe { &*thread_context };
    let object_pool = unsafe { &mut *thread_context.object_pool_pointer };
    let register_stack = unsafe { &mut *thread_context.register_vec_pointer };
    let register_tags = unsafe { &mut *thread_context.register_tag_vec_pointer };
    let register_window = &register_stack[0..thread_context.registers_used];
    let register_tags_window = &register_tags[0..thread_context.registers_used];

    let left_character = std::char::from_u32(left as u32).unwrap_or_default();
    let right_character = std::char::from_u32(right as u32).unwrap_or_default();
    let mut concatenated =
        String::with_capacity(left_character.len_utf8() + right_character.len_utf8());

    concatenated.push(left_character);
    concatenated.push(right_character);

    let object = Object::string(concatenated);

    object_pool.allocate(object, register_window, register_tags_window) as i64
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn compare_strings_equal(
    left_pointer: *mut Object,
    right_pointer: *mut Object,
) -> i8 {
    let left_string = unsafe { &*left_pointer }
        .as_string()
        .map(|string| string.as_str())
        .expect(STRING_ERROR_TEXT);
    let right_string = unsafe { &*right_pointer }
        .as_string()
        .map(|string| string.as_str())
        .expect(STRING_ERROR_TEXT);

    (left_string == right_string) as i8
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn compare_strings_less_than(
    left_pointer: *mut Object,
    right_pointer: *mut Object,
) -> i8 {
    let left_string = unsafe { &*left_pointer }
        .as_string()
        .map(|string| string.as_str())
        .expect(STRING_ERROR_TEXT);
    let right_string = unsafe { &*right_pointer }
        .as_string()
        .map(|string| string.as_str())
        .expect(STRING_ERROR_TEXT);

    (left_string < right_string) as i8
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn compare_strings_less_than_equal(
    left_pointer: *mut Object,
    right_pointer: *mut Object,
) -> i8 {
    let left_string = unsafe { &*left_pointer }
        .as_string()
        .map(|string| string.as_str())
        .expect(STRING_ERROR_TEXT);
    let right_string = unsafe { &*right_pointer }
        .as_string()
        .map(|string| string.as_str())
        .expect(STRING_ERROR_TEXT);

    (left_string <= right_string) as i8
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn integer_to_string(
    integer: i64,
    thread_context: *mut ThreadContext,
) -> i64 {
    let thread_context = unsafe { &*thread_context };
    let object_pool = unsafe { &mut *thread_context.object_pool_pointer };
    let register_stack = unsafe { &mut *thread_context.register_vec_pointer };
    let register_tags = unsafe { &mut *thread_context.register_tag_vec_pointer };
    let register_window = &register_stack[0..thread_context.registers_used];
    let register_tags_window = &register_tags[0..thread_context.registers_used];

    let object = Object::string(integer.to_string());

    object_pool.allocate(object, register_window, register_tags_window) as i64
}
