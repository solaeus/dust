use tracing::trace;

use crate::{vm::Thread, Instruction};

use super::Cache;

pub fn less_booleans(
    ip: &mut usize,
    instruction: &Instruction,
    thread: &mut Thread,
    _: &mut Cache,
) {
    let left_index = instruction.b_field() as usize;
    let right_index = instruction.c_field() as usize;
    let comparator = instruction.d_field();

    let current_frame = thread.current_frame_mut();
    let left_value = current_frame.get_boolean_from_register(left_index);
    let right_value = current_frame.get_boolean_from_register(right_index);
    let is_less_than = left_value < right_value;

    if is_less_than == comparator {
        *ip += 1;
    }
}

pub fn less_bytes(ip: &mut usize, instruction: &Instruction, thread: &mut Thread, _: &mut Cache) {
    let left = instruction.b_field() as usize;
    let right = instruction.c_field() as usize;
    let comparator = instruction.d_field();

    let current_frame = thread.current_frame_mut();
    let left_value = current_frame.get_byte_from_register(left);
    let right_value = current_frame.get_byte_from_register(right);
    let is_less_than = left_value < right_value;

    if is_less_than == comparator {
        *ip += 1;
    }
}

pub fn less_characters(
    ip: &mut usize,
    instruction: &Instruction,
    thread: &mut Thread,
    _: &mut Cache,
) {
    let left_index = instruction.b_field() as usize;
    let left_is_constant = instruction.b_is_constant();
    let right_index = instruction.c_field() as usize;
    let right_is_constant = instruction.c_is_constant();
    let comparator = instruction.d_field();

    let current_frame = thread.current_frame_mut();
    let left_value = if left_is_constant {
        current_frame.get_character_constant(left_index)
    } else {
        current_frame.get_character_from_register(left_index)
    };
    let right_value = if right_is_constant {
        current_frame.get_character_constant(right_index)
    } else {
        current_frame.get_character_from_register(right_index)
    };
    let is_less_than = left_value < right_value;

    if is_less_than == comparator {
        *ip += 1;
    }
}

pub fn less_floats(ip: &mut usize, instruction: &Instruction, thread: &mut Thread, _: &mut Cache) {
    let left = instruction.b_field() as usize;
    let left_is_constant = instruction.b_is_constant();
    let right = instruction.c_field() as usize;
    let right_is_constant = instruction.c_is_constant();
    let comparator = instruction.d_field();

    let current_frame = thread.current_frame_mut();
    let left_value = if left_is_constant {
        current_frame.get_float_constant(left)
    } else {
        current_frame.get_float_from_register(left)
    };
    let right_value = if right_is_constant {
        current_frame.get_float_constant(right)
    } else {
        current_frame.get_float_from_register(right)
    };
    let is_less_than = left_value < right_value;

    if is_less_than == comparator {
        *ip += 1;
    }
}

pub fn less_integers(
    ip: &mut usize,
    instruction: &Instruction,
    thread: &mut Thread,
    _: &mut Cache,
) {
    let left = instruction.b_field() as usize;
    let left_is_constant = instruction.b_is_constant();
    let right = instruction.c_field() as usize;
    let right_is_constant = instruction.c_is_constant();
    let comparator = instruction.d_field();

    let current_frame = thread.current_frame_mut();
    let left_value = if left_is_constant {
        current_frame.get_integer_constant(left)
    } else {
        current_frame.get_integer_from_register(left)
    };
    let right_value = if right_is_constant {
        current_frame.get_integer_constant(right)
    } else {
        current_frame.get_integer_from_register(right)
    };
    let is_less_than = left_value < right_value;

    if is_less_than == comparator {
        *ip += 1;
    }
}

pub fn less_strings(ip: &mut usize, instruction: &Instruction, thread: &mut Thread, _: &mut Cache) {
    let left = instruction.b_field() as usize;
    let left_is_constant = instruction.b_is_constant();
    let right = instruction.c_field() as usize;
    let right_is_constant = instruction.c_is_constant();
    let comparator = instruction.d_field();

    let current_frame = thread.current_frame_mut();
    let left_value = if left_is_constant {
        current_frame.get_string_constant(left)
    } else {
        current_frame.get_string_from_register(left)
    };
    let right_value = if right_is_constant {
        current_frame.get_string_constant(right)
    } else {
        current_frame.get_string_from_register(right)
    };
    let is_less_than = left_value < right_value;

    if is_less_than == comparator {
        *ip += 1;
    }
}

pub fn optimized_less_integers(
    ip: &mut usize,
    instruction: &Instruction,
    thread: &mut Thread,
    cache: &mut Cache,
) {
    if let Cache::IntegerComparison([left, right]) = cache {
        trace!("OPTIMIZED_LESS using integer cache");

        let is_less_than = left < right;

        if is_less_than {
            *ip += 1;
        }
    } else {
        let left_index = instruction.b_field() as usize;
        let left_is_constant = instruction.b_is_constant();
        let right_index = instruction.c_field() as usize;
        let right_is_constant = instruction.c_is_constant();
        let comparator = instruction.d_field();

        let current_frame = thread.current_frame_mut();
        let left_value = if left_is_constant {
            let value = current_frame.get_integer_constant_mut(left_index).to_rc();

            current_frame.constants.integers[left_index] = value.clone();

            value
        } else {
            let value = current_frame
                .get_integer_from_register_mut(left_index)
                .to_ref_cell();

            current_frame.registers.integers[left_index].set(value.clone());

            value
        };
        let right_value = if right_is_constant {
            let value = current_frame.get_integer_constant_mut(right_index).to_rc();

            current_frame.constants.integers[right_index] = value.clone();

            value
        } else {
            let value = current_frame
                .get_integer_from_register_mut(right_index)
                .to_ref_cell();

            current_frame.registers.integers[right_index].set(value.clone());

            value
        };
        let is_less_than = left_value < right_value;

        if is_less_than == comparator {
            *ip += 1;
        }

        *cache = Cache::IntegerComparison([left_value, right_value]);
    }
}
