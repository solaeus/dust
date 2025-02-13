use tracing::trace;

use crate::{
    instruction::{InstructionFields, TypeCode},
    vm::{call_frame::PointerCache, Register, Thread},
    DustString,
};

pub fn add_integers(ip: &mut usize, instruction: &InstructionFields, thread: &mut Thread) {
    trace!("ADD: Run and cache pointers");

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
