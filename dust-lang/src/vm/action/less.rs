use crate::{instruction::InstructionFields, vm::Thread};

pub fn less_booleans(ip: &mut usize, instruction: &InstructionFields, thread: &mut Thread) {
    let left = instruction.b_field as usize;
    let left_is_constant = instruction.b_is_constant;
    let right = instruction.c_field as usize;
    let right_is_constant = instruction.c_is_constant;
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
    let is_less_than = left_value < right_value;
    let comparator = instruction.d_field;

    if is_less_than == comparator {
        *ip += 1;
    }
}

pub fn less_bytes(ip: &mut usize, instruction: &InstructionFields, thread: &mut Thread) {
    let left = instruction.b_field as usize;
    let left_is_constant = instruction.b_is_constant;
    let right = instruction.c_field as usize;
    let right_is_constant = instruction.c_is_constant;
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
    let is_less_than = left_value < right_value;
    let comparator = instruction.d_field;

    if is_less_than == comparator {
        *ip += 1;
    }
}

pub fn less_characters(ip: &mut usize, instruction: &InstructionFields, thread: &mut Thread) {
    let left = instruction.b_field as usize;
    let left_is_constant = instruction.b_is_constant;
    let right = instruction.c_field as usize;
    let right_is_constant = instruction.c_is_constant;
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
    let is_less_than = left_value < right_value;
    let comparator = instruction.d_field;

    if is_less_than == comparator {
        *ip += 1;
    }
}

pub fn less_floats(ip: &mut usize, instruction: &InstructionFields, thread: &mut Thread) {
    let left = instruction.b_field as usize;
    let left_is_constant = instruction.b_is_constant;
    let right = instruction.c_field as usize;
    let right_is_constant = instruction.c_is_constant;
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
    let is_less_than = left_value < right_value;
    let comparator = instruction.d_field;

    if is_less_than == comparator {
        *ip += 1;
    }
}

pub fn less_integers(ip: &mut usize, instruction: &InstructionFields, thread: &mut Thread) {
    let left = instruction.b_field as usize;
    let left_is_constant = instruction.b_is_constant;
    let right = instruction.c_field as usize;
    let right_is_constant = instruction.c_is_constant;
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
    let is_less_than = left_value < right_value;
    let comparator = instruction.d_field;

    if is_less_than == comparator {
        *ip += 1;
    }
}

pub fn less_strings(ip: &mut usize, instruction: &InstructionFields, thread: &mut Thread) {
    let left = instruction.b_field as usize;
    let left_is_constant = instruction.b_is_constant;
    let right = instruction.c_field as usize;
    let right_is_constant = instruction.c_is_constant;
    let left_value = if left_is_constant {
        if cfg!(debug_assertions) {
            thread.get_constant(left).as_string().unwrap()
        } else {
            unsafe { thread.get_constant(left).as_string().unwrap_unchecked() }
        }
    } else {
        thread.get_string_register(left)
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
    let is_less_than = left_value < right_value;
    let comparator = instruction.d_field;

    if is_less_than == comparator {
        *ip += 1;
    }
}
