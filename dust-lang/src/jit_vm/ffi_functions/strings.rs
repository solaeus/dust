use std::{ptr, slice};

use crate::jit_vm::{
    ERROR_REPLACEMENT_STR, Object, object_pool::ObjectIndex, thread_pool::ThreadContext,
};

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
    let object_index = object_pool.allocate(object, register_window, register_tags_window);

    object_index.encode() as i64
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn concatenate_strings(
    encoded_left_index: i64,
    enoded_right_index: i64,
    thread_context: *mut ThreadContext,
) -> i64 {
    let left_index = ObjectIndex::decode(encoded_left_index as u64);
    let right_index = ObjectIndex::decode(enoded_right_index as u64);
    let thread_context = unsafe { &*thread_context };
    let object_pool = unsafe { &mut *thread_context.object_pool_pointer };
    let register_stack = unsafe { &mut *thread_context.register_vec_pointer };
    let register_tags = unsafe { &mut *thread_context.register_tag_vec_pointer };
    let register_window = &register_stack[0..thread_context.registers_used];
    let register_tags_window = &register_tags[0..thread_context.registers_used];

    let left = object_pool.get(left_index).expect("List object not found");
    let right = object_pool.get(right_index).expect("List object not found");

    let concatenated = if ptr::eq(left, right) {
        let right_string = right
            .as_string()
            .cloned()
            .unwrap_or_else(|| ERROR_REPLACEMENT_STR.to_string());
        let left_string = left
            .as_string()
            .map(|string| string.as_str())
            .unwrap_or(ERROR_REPLACEMENT_STR);
        let mut concatenated = String::with_capacity(left_string.len() + right_string.len());

        concatenated.push_str(left_string);
        concatenated.push_str(&right_string);

        concatenated
    } else {
        let left_string = left
            .as_string()
            .map(|string| string.as_str())
            .unwrap_or(ERROR_REPLACEMENT_STR);
        let right_string = right
            .as_string()
            .map(|string| string.as_str())
            .unwrap_or(ERROR_REPLACEMENT_STR);
        let mut concatenated = String::with_capacity(left_string.len() + right_string.len());

        concatenated.push_str(left_string);
        concatenated.push_str(right_string);

        concatenated
    };
    let object = Object::string(concatenated);
    let object_index = object_pool.allocate(object, register_window, register_tags_window);

    object_index.encode() as i64
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn concatenate_character_string(
    character: i64,
    encoded_object_index: i64,
    thread_context: *mut ThreadContext,
) -> i64 {
    let object_index = ObjectIndex::decode(encoded_object_index as u64);
    let thread_context = unsafe { &*thread_context };
    let object_pool = unsafe { &mut *thread_context.object_pool_pointer };
    let register_stack = unsafe { &mut *thread_context.register_vec_pointer };
    let register_tags = unsafe { &mut *thread_context.register_tag_vec_pointer };
    let register_window = &register_stack[0..thread_context.registers_used];
    let register_tags_window = &register_tags[0..thread_context.registers_used];

    let left_character = unsafe { char::from_u32_unchecked(character as u32) };
    let right_string = object_pool
        .get(object_index)
        .and_then(|object| object.as_string().map(|string| string.as_str()))
        .unwrap_or(ERROR_REPLACEMENT_STR);
    let mut concatenated = String::with_capacity(left_character.len_utf8() + right_string.len());

    concatenated.push(left_character);
    concatenated.push_str(right_string);

    let object = Object::string(concatenated);
    let object_index = object_pool.allocate(object, register_window, register_tags_window);

    object_index.encode() as i64
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn concatenate_string_character(
    encoded_object_index: i64,
    character: i64,
    thread_context: *mut ThreadContext,
) -> i64 {
    let object_index = ObjectIndex::decode(encoded_object_index as u64);
    let thread_context = unsafe { &*thread_context };
    let object_pool = unsafe { &mut *thread_context.object_pool_pointer };
    let register_stack = unsafe { &mut *thread_context.register_vec_pointer };
    let register_tags = unsafe { &mut *thread_context.register_tag_vec_pointer };
    let register_window = &register_stack[0..thread_context.registers_used];
    let register_tags_window = &register_tags[0..thread_context.registers_used];

    let left_string = object_pool
        .get(object_index)
        .and_then(|object| object.as_string().map(|string| string.as_str()))
        .unwrap_or(ERROR_REPLACEMENT_STR);
    let right_character = unsafe { char::from_u32_unchecked(character as u32) };
    let mut concatenated = String::with_capacity(left_string.len() + right_character.len_utf8());

    concatenated.push_str(left_string);
    concatenated.push(right_character);

    let object = Object::string(concatenated);
    let object_index = object_pool.allocate(object, register_window, register_tags_window);

    object_index.encode() as i64
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
    let object_index = object_pool.allocate(object, register_window, register_tags_window);

    object_index.encode() as i64
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn compare_strings_equal(
    encoded_left_index: i64,
    enoded_right_index: i64,
    thread_context: *mut ThreadContext,
) -> i8 {
    let left_index = ObjectIndex::decode(encoded_left_index as u64);
    let right_index = ObjectIndex::decode(enoded_right_index as u64);
    let thread_context = unsafe { &*thread_context };
    let object_pool = unsafe { &mut *thread_context.object_pool_pointer };

    let left_string = object_pool
        .get(left_index)
        .and_then(|object| object.as_string().map(|string| string.as_str()))
        .unwrap_or(ERROR_REPLACEMENT_STR);
    let right_string = object_pool
        .get(right_index)
        .and_then(|object| object.as_string().map(|string| string.as_str()))
        .unwrap_or(ERROR_REPLACEMENT_STR);

    (left_string == right_string) as i8
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn compare_strings_less_than(
    encoded_left_index: i64,
    enoded_right_index: i64,
    thread_context: *mut ThreadContext,
) -> i8 {
    let left_index = ObjectIndex::decode(encoded_left_index as u64);
    let right_index = ObjectIndex::decode(enoded_right_index as u64);
    let thread_context = unsafe { &*thread_context };
    let object_pool = unsafe { &mut *thread_context.object_pool_pointer };

    let left_string = object_pool
        .get(left_index)
        .and_then(|object| object.as_string().map(|string| string.as_str()))
        .unwrap_or(ERROR_REPLACEMENT_STR);
    let right_string = object_pool
        .get(right_index)
        .and_then(|object| object.as_string().map(|string| string.as_str()))
        .unwrap_or(ERROR_REPLACEMENT_STR);

    (left_string < right_string) as i8
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn compare_strings_less_than_equal(
    encoded_left_index: i64,
    enoded_right_index: i64,
    thread_context: *mut ThreadContext,
) -> i8 {
    let left_index = ObjectIndex::decode(encoded_left_index as u64);
    let right_index = ObjectIndex::decode(enoded_right_index as u64);
    let thread_context = unsafe { &*thread_context };
    let object_pool = unsafe { &mut *thread_context.object_pool_pointer };

    let left_string = object_pool
        .get(left_index)
        .and_then(|object| object.as_string().map(|string| string.as_str()))
        .unwrap_or(ERROR_REPLACEMENT_STR);
    let right_string = object_pool
        .get(right_index)
        .and_then(|object| object.as_string().map(|string| string.as_str()))
        .unwrap_or(ERROR_REPLACEMENT_STR);

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
    let object_index = object_pool.allocate(object, register_window, register_tags_window);

    object_index.encode() as i64
}
