use crate::{
    instruction::InstructionFields,
    vm::{Register, Thread},
    DustString,
};

pub fn add_bytes(_: &mut usize, instruction: &InstructionFields, thread: &mut Thread) {
    let destination = instruction.a_field as usize;
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
    let sum = left_value.saturating_add(*right_value);

    thread.set_byte_register(destination, Register::Value(sum));
}

pub fn add_characters(_: &mut usize, instruction: &InstructionFields, thread: &mut Thread) {
    let destination = instruction.a_field as usize;
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
    let mut concatenated = DustString::from(String::with_capacity(2));

    concatenated.push(*left_value);
    concatenated.push(*right_value);

    thread.set_string_register(destination, Register::Value(concatenated));
}

pub fn add_floats(_: &mut usize, instruction: &InstructionFields, thread: &mut Thread) {
    let destination = instruction.a_field as usize;
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
    let sum = left_value + *right_value;

    thread.set_float_register(destination, Register::Value(sum));
}

pub fn add_integers(_: &mut usize, instruction: &InstructionFields, thread: &mut Thread) {
    let destination = instruction.a_field as usize;
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
    let sum = left_value.saturating_add(*right_value);

    thread.set_integer_register(destination, Register::Value(sum));
}

pub fn add_strings(_: &mut usize, instruction: &InstructionFields, thread: &mut Thread) {
    let destination = instruction.a_field as usize;
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
    let mut concatenated =
        DustString::from(String::with_capacity(left_value.len() + right_value.len()));

    concatenated.push_str(left_value);
    concatenated.push_str(right_value);

    thread.set_string_register(destination, Register::Value(concatenated));
}

pub fn add_character_string(_: &mut usize, instruction: &InstructionFields, thread: &mut Thread) {
    let destination = instruction.a_field as usize;
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
            thread.get_constant(right).as_string().unwrap()
        } else {
            unsafe { thread.get_constant(right).as_string().unwrap_unchecked() }
        }
    } else {
        thread.get_string_register(right)
    };
    let mut concatenated = DustString::from(String::with_capacity(right_value.len() + 1));

    concatenated.push(*left_value);
    concatenated.push_str(right_value);

    thread.set_string_register(destination, Register::Value(concatenated));
}

pub fn add_string_character(_: &mut usize, instruction: &InstructionFields, thread: &mut Thread) {
    let destination = instruction.a_field as usize;
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
            thread.get_constant(right).as_character().unwrap()
        } else {
            unsafe { thread.get_constant(right).as_character().unwrap_unchecked() }
        }
    } else {
        thread.get_character_register(right)
    };
    let mut concatenated = DustString::from(String::with_capacity(left_value.len() + 1));

    concatenated.push_str(left_value);
    concatenated.push(*right_value);

    thread.set_string_register(destination, Register::Value(concatenated));
}
