use tracing::trace;

use crate::{
    DustString,
    instruction::{InstructionFields, TypeCode},
    vm::{Register, Thread, call_frame::PointerCache},
};

pub fn add(
    _: &mut usize,
    instruction: InstructionFields,
    thread: &mut Thread,
    pointer_cache: &mut PointerCache,
) {
    let destination = instruction.a_field as usize;
    let left = instruction.b_field as usize;
    let left_is_constant = instruction.b_is_constant;
    let left_type = instruction.b_type;
    let right = instruction.c_field as usize;
    let right_is_constant = instruction.c_is_constant;
    let right_type = instruction.c_type;

    match (left_type, right_type) {
        (TypeCode::INTEGER, TypeCode::INTEGER) => add_integers(instruction, thread, pointer_cache),
        (TypeCode::BYTE, TypeCode::BYTE) => {
            let left_value = if left_is_constant {
                if cfg!(debug_assertions) {
                    thread.get_constant(left).as_byte().unwrap()
                } else {
                    unsafe { thread.get_constant(left).as_byte().unwrap_unchecked() }
                }
            } else {
                thread.get_byte_register(left)
            };
            let right_value = if right_is_constant {
                if cfg!(debug_assertions) {
                    thread.get_constant(right).as_byte().unwrap()
                } else {
                    unsafe { thread.get_constant(right).as_byte().unwrap_unchecked() }
                }
            } else {
                thread.get_byte_register(right)
            };
            let sum = left_value + right_value;
            let register = Register::Value(sum);

            thread.set_byte_register(destination, register);
        }
        (TypeCode::FLOAT, TypeCode::FLOAT) => {
            let left_value = if left_is_constant {
                if cfg!(debug_assertions) {
                    thread.get_constant(left).as_float().unwrap()
                } else {
                    unsafe { thread.get_constant(left).as_float().unwrap_unchecked() }
                }
            } else {
                thread.get_float_register(left)
            };
            let right_value = if right_is_constant {
                if cfg!(debug_assertions) {
                    thread.get_constant(right).as_float().unwrap()
                } else {
                    unsafe { thread.get_constant(right).as_float().unwrap_unchecked() }
                }
            } else {
                thread.get_float_register(right)
            };
            let sum = left_value + right_value;
            let register = Register::Value(sum);

            thread.set_float_register(destination, register);
        }
        (TypeCode::STRING, TypeCode::STRING) => {
            let left_value = if left_is_constant {
                if cfg!(debug_assertions) {
                    thread.get_constant(left).as_string().unwrap().clone()
                } else {
                    unsafe {
                        thread
                            .get_constant(left)
                            .as_string()
                            .unwrap_unchecked()
                            .clone()
                    }
                }
            } else {
                thread.get_string_register(left).clone()
            };
            let right_value = if right_is_constant {
                if cfg!(debug_assertions) {
                    thread.get_constant(right).as_string().unwrap().clone()
                } else {
                    unsafe {
                        thread
                            .get_constant(right)
                            .as_string()
                            .unwrap_unchecked()
                            .clone()
                    }
                }
            } else {
                thread.get_string_register(right).clone()
            };
            let concatenated = left_value + &right_value;
            let register = Register::Value(concatenated);

            thread.set_string_register(destination, register);
        }
        (TypeCode::CHARACTER, TypeCode::CHARACTER) => {
            let left_value = if left_is_constant {
                if cfg!(debug_assertions) {
                    thread.get_constant(left).as_character().unwrap()
                } else {
                    unsafe { thread.get_constant(left).as_character().unwrap_unchecked() }
                }
            } else {
                thread.get_character_register(left)
            };
            let right_value = if right_is_constant {
                if cfg!(debug_assertions) {
                    thread.get_constant(right).as_character().unwrap()
                } else {
                    unsafe { thread.get_constant(right).as_character().unwrap_unchecked() }
                }
            } else {
                thread.get_character_register(right)
            };
            let mut sum = DustString::new();

            sum.push(*left_value);
            sum.push(*right_value);

            let register = Register::Value(sum);

            thread.set_string_register(destination, register);
        }
        (TypeCode::STRING, TypeCode::CHARACTER) => {
            let left_value = if left_is_constant {
                if cfg!(debug_assertions) {
                    thread.get_constant(left).as_string().unwrap().clone()
                } else {
                    unsafe {
                        thread
                            .get_constant(left)
                            .as_string()
                            .unwrap_unchecked()
                            .clone()
                    }
                }
            } else {
                thread.get_string_register(left).clone()
            };
            let right_value = if right_is_constant {
                if cfg!(debug_assertions) {
                    thread.get_constant(right).as_character().unwrap()
                } else {
                    unsafe { thread.get_constant(right).as_character().unwrap_unchecked() }
                }
            } else {
                thread.get_character_register(right)
            };
            let mut sum = left_value.clone();

            sum.push(*right_value);

            let register = Register::Value(sum);

            thread.set_string_register(destination, register);
        }
        (TypeCode::CHARACTER, TypeCode::STRING) => {
            let left_value = if left_is_constant {
                if cfg!(debug_assertions) {
                    thread.get_constant(left).as_character().unwrap()
                } else {
                    unsafe { thread.get_constant(left).as_character().unwrap_unchecked() }
                }
            } else {
                thread.get_character_register(left)
            };
            let right_value = if right_is_constant {
                if cfg!(debug_assertions) {
                    thread.get_constant(right).as_string().unwrap()
                } else {
                    unsafe { thread.get_constant(right).as_string().unwrap_unchecked() }
                }
            } else {
                thread.get_string_register(right)
            };
            let mut sum = right_value.clone();

            sum.insert(0, *left_value);

            let register = Register::Value(sum);

            thread.set_string_register(destination, register);
        }
        _ => unreachable!(),
    }
}

pub fn add_integers(
    instruction: InstructionFields,
    thread: &mut Thread,
    pointer_cache: &mut PointerCache,
) {
    let destination = instruction.a_field as usize;
    let left = instruction.b_field as usize;
    let left_is_constant = instruction.b_is_constant;
    let right = instruction.c_field as usize;
    let right_is_constant = instruction.c_is_constant;

    if pointer_cache.integer_mut.is_null() {
        trace!("ADD: Run and cache pointers");

        let left_value = if left_is_constant {
            if cfg!(debug_assertions) {
                thread.get_constant(left).as_integer().unwrap()
            } else {
                unsafe { thread.get_constant(left).as_integer().unwrap_unchecked() }
            }
        } else {
            thread.get_integer_register(left)
        };
        let right_value = if right_is_constant {
            if cfg!(debug_assertions) {
                thread.get_constant(right).as_integer().unwrap()
            } else {
                unsafe { thread.get_constant(right).as_integer().unwrap_unchecked() }
            }
        } else {
            thread.get_integer_register(right)
        };
        let sum = left_value.saturating_add(*right_value);

        pointer_cache.integer_left = left_value;
        pointer_cache.integer_right = right_value;
        pointer_cache.integer_mut = thread.get_integer_register_mut_allow_empty(destination);

        thread.set_integer_register(destination, Register::Value(sum));
    } else {
        trace!("ADD: Use cached pointers");

        add_integer_pointers(
            pointer_cache.integer_mut,
            pointer_cache.integer_left,
            pointer_cache.integer_right,
        );
    };
}

pub fn add_integer_pointers(destination: *mut i64, left: *const i64, right: *const i64) {
    assert!(destination.is_aligned());
    assert!(left.is_aligned());
    assert!(right.is_aligned());

    unsafe {
        *destination = (*left).saturating_add(*right);
    }
}
