use tracing::trace;

use crate::{
    instruction::{InstructionFields, TypeCode},
    vm::{Thread, call_frame::PointerCache},
};

pub fn less(
    ip: &mut usize,
    instruction: InstructionFields,
    thread: &mut Thread,
    pointer_cache: &mut PointerCache,
) {
    let comparator = instruction.d_field;
    let left = instruction.b_field as usize;
    let left_type = instruction.b_type;
    let left_is_constant = instruction.b_is_constant;
    let right = instruction.c_field as usize;
    let right_type = instruction.c_type;
    let right_is_constant = instruction.c_is_constant;

    match (left_type, right_type) {
        (TypeCode::BOOLEAN, TypeCode::BOOLEAN) => {
            let left_value = if left_is_constant {
                if cfg!(debug_assertions) {
                    thread.get_constant(left).as_boolean().unwrap()
                } else {
                    unsafe { thread.get_constant(left).as_boolean().unwrap_unchecked() }
                }
            } else {
                thread.get_boolean_register(left)
            };
            let right_value = if right_is_constant {
                if cfg!(debug_assertions) {
                    thread.get_constant(right).as_boolean().unwrap()
                } else {
                    unsafe { thread.get_constant(right).as_boolean().unwrap_unchecked() }
                }
            } else {
                thread.get_boolean_register(right)
            };
            let result = left_value < right_value;

            if result == comparator {
                *ip += 1;
            }
        }
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
            let result = left_value < right_value;

            if result == comparator {
                *ip += 1;
            }
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
            let result = left_value < right_value;

            if result == comparator {
                *ip += 1;
            }
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
            let result = left_value < right_value;

            if result == comparator {
                *ip += 1;
            }
        }
        (TypeCode::INTEGER, TypeCode::INTEGER) => {
            less_integers(ip, instruction, thread, pointer_cache)
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
            let result = left_value < right_value;

            if result == comparator {
                *ip += 1;
            }
        }
        _ => unreachable!(),
    }
}

pub fn less_integers(
    ip: &mut usize,
    instruction: InstructionFields,
    thread: &mut Thread,
    pointer_cache: &mut PointerCache,
) {
    if pointer_cache.integer_left.is_null() {
        trace!("LESS: Run and cache pointers");

        let left = instruction.b_field as usize;
        let left_is_constant = instruction.b_is_constant;
        let right = instruction.c_field as usize;
        let right_is_constant = instruction.c_is_constant;
        let left_pointer = if left_is_constant {
            if cfg!(debug_assertions) {
                thread.get_constant(left).as_integer().unwrap()
            } else {
                unsafe { thread.get_constant(left).as_integer().unwrap_unchecked() }
            }
        } else {
            thread.get_integer_register(left)
        } as *const i64;
        let right_pointer = if right_is_constant {
            if cfg!(debug_assertions) {
                thread.get_constant(right).as_integer().unwrap()
            } else {
                unsafe { thread.get_constant(right).as_integer().unwrap_unchecked() }
            }
        } else {
            thread.get_integer_register(right)
        } as *const i64;

        pointer_cache.integer_left = left_pointer;
        pointer_cache.integer_right = right_pointer;

        less_integer_pointers(ip, left_pointer, right_pointer, instruction.d_field);
    } else {
        trace!("LESS: Use cached pointers");

        less_integer_pointers(
            ip,
            pointer_cache.integer_left,
            pointer_cache.integer_right,
            instruction.d_field,
        );
    };
}

pub fn less_integer_pointers(
    ip: *mut usize,
    left: *const i64,
    right: *const i64,
    comparator: bool,
) {
    assert!(ip.is_aligned());
    assert!(left.is_aligned());
    assert!(right.is_aligned());

    unsafe {
        let is_less_than = *left < *right;

        if is_less_than == comparator {
            *ip += 1;
        }
    }
}
