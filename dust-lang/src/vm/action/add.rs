use std::ops::Add;

use tracing::trace;

use crate::{
    instruction::InstructionFields,
    vm::{call_frame::RuntimeValue, Thread},
    DustString,
};

pub fn add_bytes(_: &mut usize, instruction: &InstructionFields, thread: &mut Thread) {
    let destination_index = instruction.a_field as usize;
    let left_index = instruction.b_field as usize;
    let right_index = instruction.c_field as usize;

    let current_frame = thread.current_frame_mut();
    let left_value = current_frame.get_byte_from_register(left_index);
    let right_value = current_frame.get_byte_from_register(right_index);
    let sum = left_value.add(right_value);

    current_frame
        .registers
        .bytes
        .get_mut(destination_index)
        .as_value_mut()
        .set_inner(sum);
}

pub fn add_characters(_: &mut usize, instruction: &InstructionFields, thread: &mut Thread) {
    let destination_index = instruction.a_field as usize;
    let left_index = instruction.b_field as usize;
    let left_is_constant = instruction.b_is_constant;
    let right = instruction.c_field as usize;
    let right_is_constant = instruction.c_is_constant;

    let current_frame = thread.current_frame_mut();
    let left_value = if left_is_constant {
        current_frame.get_character_constant(left_index)
    } else {
        current_frame.get_character_from_register(left_index)
    };
    let right_value = if right_is_constant {
        current_frame.get_character_constant(right)
    } else {
        current_frame.get_character_from_register(right)
    };
    let concatenated = {
        let mut concatenated = DustString::from(String::with_capacity(2));

        concatenated.push(left_value.clone_inner());
        concatenated.push(right_value.clone_inner());

        RuntimeValue::Raw(concatenated)
    };

    current_frame
        .registers
        .strings
        .get_mut(destination_index)
        .set(concatenated);
}

pub fn add_floats(_: &mut usize, instruction: &InstructionFields, thread: &mut Thread) {
    let destination = instruction.a_field as usize;
    let left = instruction.b_field as usize;
    let left_is_constant = instruction.b_is_constant;
    let right = instruction.c_field as usize;
    let right_is_constant = instruction.c_is_constant;

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
    let sum = left_value.add(right_value);

    current_frame
        .registers
        .floats
        .get_mut(destination)
        .as_value_mut()
        .set_inner(sum);
}

pub fn add_integers(_: &mut usize, instruction: &InstructionFields, thread: &mut Thread) {
    let destination = instruction.a_field as usize;
    let left = instruction.b_field as usize;
    let left_is_constant = instruction.b_is_constant;
    let right = instruction.c_field as usize;
    let right_is_constant = instruction.c_is_constant;

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
    let sum = left_value.add(right_value);

    current_frame
        .registers
        .integers
        .get_mut(destination)
        .as_value_mut()
        .set_inner(sum);
}

pub fn add_strings(_: &mut usize, instruction: &InstructionFields, thread: &mut Thread) {
    let destination = instruction.a_field as usize;
    let left = instruction.b_field as usize;
    let left_is_constant = instruction.b_is_constant;
    let right = instruction.c_field as usize;
    let right_is_constant = instruction.c_is_constant;

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
    let concatenated = DustString::from(format!("{left_value}{right_value}"));

    current_frame
        .registers
        .strings
        .get_mut(destination)
        .as_value_mut()
        .set_inner(concatenated);
}

pub fn add_character_string(_: &mut usize, instruction: &InstructionFields, thread: &mut Thread) {
    let destination = instruction.a_field as usize;
    let left = instruction.b_field as usize;
    let left_is_constant = instruction.b_is_constant;
    let right = instruction.c_field as usize;
    let right_is_constant = instruction.c_is_constant;

    let current_frame = thread.current_frame_mut();
    let left_value = if left_is_constant {
        current_frame.get_character_constant(left)
    } else {
        current_frame.get_character_from_register(left)
    };
    let right_value = if right_is_constant {
        current_frame.get_string_constant(right)
    } else {
        current_frame.get_string_from_register(right)
    };
    let concatenated = DustString::from(format!("{left_value}{right_value}"));

    current_frame
        .registers
        .strings
        .get_mut(destination)
        .as_value_mut()
        .set_inner(concatenated);
}

pub fn add_string_character(_: &mut usize, instruction: &InstructionFields, thread: &mut Thread) {
    let destination = instruction.a_field as usize;
    let left = instruction.b_field as usize;
    let left_is_constant = instruction.b_is_constant;
    let right = instruction.c_field as usize;
    let right_is_constant = instruction.c_is_constant;

    let current_frame = thread.current_frame_mut();
    let left_value = if left_is_constant {
        current_frame.get_string_constant(left)
    } else {
        current_frame.get_string_from_register(left)
    };
    let right_value = if right_is_constant {
        current_frame.get_character_constant(right)
    } else {
        current_frame.get_character_from_register(right)
    };
    let concatenated = DustString::from(format!("{left_value}{right_value}"));

    current_frame
        .registers
        .strings
        .get_mut(destination)
        .as_value_mut()
        .set_inner(concatenated);
}

pub fn optimized_add_integer(
    instruction: &InstructionFields,
    thread: &mut Thread,
    cache: &mut Option<[RuntimeValue<i64>; 3]>,
) {
    if let Some([destination, left, right]) = cache {
        trace!("ADD_INTEGERS_OPTIMIZED using cache");

        let sum = left.add(right);

        *destination.borrow_mut() = sum;
    } else {
        let destination_index = instruction.a_field as usize;
        let left_index = instruction.b_field as usize;
        let left_is_constant = instruction.b_is_constant;
        let right_index = instruction.c_field as usize;
        let right_is_constant = instruction.c_is_constant;
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
        let sum = left_value.add(&right_value);
        let destination = {
            let mut value = current_frame
                .get_integer_from_register_mut(destination_index)
                .to_ref_cell();

            value.set_inner(sum);

            current_frame.registers.integers[destination_index].set(value.clone());

            value
        };

        *cache = Some([destination, left_value, right_value]);
    }
}
