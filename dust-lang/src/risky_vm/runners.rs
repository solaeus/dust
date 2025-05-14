use tracing::warn;

use crate::{
    ConcreteValue, Instruction,
    instruction::{Add, AddressKind, Jump, Less, LoadConstant, Move, Return},
};

use super::{RegisterTable, Thread};

pub type Runner = fn(Instruction, &mut Thread, &mut RegisterTable);

pub fn run_no_op(_: Instruction, _: &mut Thread, registers: &mut RegisterTable) {
    warn!("Running NO_OP instruction");
}

pub fn run_move(instruction: Instruction, thread: &mut Thread, mut registers: &mut RegisterTable) {
    let Move {
        destination: to,
        operand: from,
    } = Move::from(&instruction);

    match from.kind {
        AddressKind::BOOLEAN_MEMORY => {
            let boolean = *thread
                .current_memory
                .booleans
                .get(from.index as usize)
                .unwrap()
                .as_value();

            *thread
                .current_memory
                .booleans
                .get_mut(to.index as usize)
                .unwrap()
                .as_value_mut() = boolean;
        }
        AddressKind::BOOLEAN_REGISTER => {
            let boolean = *registers.booleans.get(from.index);

            registers.booleans.set(to.index, boolean);
        }
        _ => unimplemented!(),
    }
}

pub fn run_close(instruction: Instruction, thread: &mut Thread, registers: &mut RegisterTable) {
    todo!()
}

pub fn run_load_encoded(
    instruction: Instruction,
    thread: &mut Thread,
    registers: &mut RegisterTable,
) {
    todo!()
}

pub fn run_load_constant(
    instruction: Instruction,
    thread: &mut Thread,
    mut registers: &mut RegisterTable,
) {
    let LoadConstant {
        destination,
        constant,
        jump_next,
    } = LoadConstant::from(&instruction);
    let constant_index = constant.index as usize;

    match constant.kind {
        AddressKind::CHARACTER_CONSTANT => {
            let value = thread.chunk.character_constants[constant_index];

            if destination.is_register {
                registers.characters.set(destination.index, value);
            } else {
                let destination_index = destination.index as usize;

                *thread.current_memory.characters[destination_index].as_value_mut() = value;
            }
        }
        AddressKind::FLOAT_CONSTANT => {
            let value = thread.chunk.float_constants[constant_index];

            if destination.is_register {
                registers.floats.set(destination.index, value);
            } else {
                let destination_index = destination.index as usize;

                *thread.current_memory.floats[destination_index].as_value_mut() = value;
            }
        }
        AddressKind::INTEGER_CONSTANT => {
            let value = thread.chunk.integer_constants[constant_index];

            if destination.is_register {
                registers.integers.set(destination.index, value);
            } else {
                let destination_index = destination.index as usize;

                *thread.current_memory.integers[destination_index].as_value_mut() = value;
            }
        }
        AddressKind::STRING_CONSTANT => {
            let value = thread.chunk.string_constants[constant_index].clone();

            if destination.is_register {
                registers.strings.set(destination.index, value);
            } else {
                let destination_index = destination.index as usize;

                *thread.current_memory.strings[destination_index].as_value_mut() = value;
            }
        }
        _ => unreachable!(),
    };

    if jump_next {
        thread.current_call.ip += 1;
    }
}

pub fn run_load_list(instruction: Instruction, thread: &mut Thread, registers: &mut RegisterTable) {
    todo!()
}

pub fn run_load_function(
    instruction: Instruction,
    thread: &mut Thread,
    registers: &mut RegisterTable,
) {
    todo!()
}

pub fn run_add(instruction: Instruction, thread: &mut Thread, mut registers: &mut RegisterTable) {
    let Add {
        destination,
        left,
        right,
    } = Add::from(&instruction);

    match left.kind {
        AddressKind::INTEGER_CONSTANT => {
            let left_value = thread.chunk.integer_constants[left.index as usize];
            let right_value = thread.resolve_integer(&right, &registers);
            let sum = left_value + right_value;

            if destination.is_register {
                registers.integers.set(destination.index, sum);
            } else {
                *thread.current_memory.integers[destination.index as usize].as_value_mut() = sum;
            }
        }
        AddressKind::INTEGER_MEMORY => {
            let left_value = thread.current_memory.integers[left.index as usize].as_value();
            let right_value = thread.resolve_integer(&right, &registers);
            let sum = left_value + right_value;

            if destination.is_register {
                registers.integers.set(destination.index, sum);
            } else {
                *thread.current_memory.integers[destination.index as usize].as_value_mut() = sum;
            }
        }
        AddressKind::INTEGER_REGISTER => {
            let left_value = registers.integers.get(left.index);
            let right_value = thread.resolve_integer(&right, &registers);
            let sum = left_value + right_value;

            if destination.is_register {
                registers.integers.set(destination.index, sum);
            } else {
                *thread.current_memory.integers[destination.index as usize].as_value_mut() = sum;
            }
        }
        _ => todo!(),
    }
}

pub fn run_subtract(instruction: Instruction, thread: &mut Thread, registers: &mut RegisterTable) {
    todo!()
}

pub fn run_multiply(instruction: Instruction, thread: &mut Thread, registers: &mut RegisterTable) {
    todo!()
}

pub fn run_divide(instruction: Instruction, thread: &mut Thread, registers: &mut RegisterTable) {
    todo!()
}

pub fn run_modulo(instruction: Instruction, thread: &mut Thread, registers: &mut RegisterTable) {
    todo!()
}

pub fn run_equal(instruction: Instruction, thread: &mut Thread, registers: &mut RegisterTable) {
    todo!()
}

pub fn run_less(instruction: Instruction, thread: &mut Thread, registers: &mut RegisterTable) {
    let Less {
        comparator,
        left,
        right,
    } = Less::from(&instruction);

    let is_less_than = match left.kind {
        AddressKind::INTEGER_MEMORY => {
            let left_value = thread.current_memory.integers[left.index as usize].as_value();
            let right_value = thread.resolve_integer(&right, &registers);

            left_value < right_value
        }
        AddressKind::INTEGER_REGISTER => {
            let left_value = registers.integers.get(left.index);
            let right_value = thread.resolve_integer(&right, &registers);

            left_value < right_value
        }
        _ => todo!(),
    };

    if is_less_than != comparator {
        thread.current_call.ip += 1;
    }
}

pub fn run_less_equal(
    instruction: Instruction,
    thread: &mut Thread,
    registers: &mut RegisterTable,
) {
    todo!()
}

pub fn run_negate(instruction: Instruction, thread: &mut Thread, registers: &mut RegisterTable) {
    todo!()
}

pub fn run_not(instruction: Instruction, thread: &mut Thread, registers: &mut RegisterTable) {
    todo!()
}

pub fn run_test(instruction: Instruction, thread: &mut Thread, registers: &mut RegisterTable) {
    todo!()
}

pub fn run_test_set(instruction: Instruction, thread: &mut Thread, registers: &mut RegisterTable) {
    todo!()
}

pub fn run_call(instruction: Instruction, thread: &mut Thread, registers: &mut RegisterTable) {
    todo!()
}

pub fn run_call_native(
    instruction: Instruction,
    thread: &mut Thread,
    registers: &mut RegisterTable,
) {
    todo!()
}

pub fn run_jump(instruction: Instruction, thread: &mut Thread, registers: &mut RegisterTable) {
    let Jump {
        offset,
        is_positive,
    } = Jump::from(&instruction);

    if is_positive {
        thread.current_call.ip += offset as usize;
    } else {
        thread.current_call.ip -= (offset + 1) as usize;
    }
}

pub fn run_return(instruction: Instruction, thread: &mut Thread, registers: &mut RegisterTable) {
    let Return {
        should_return_value,
        return_address,
    } = Return::from(&instruction);

    if should_return_value {
        match return_address.kind {
            AddressKind::INTEGER_REGISTER => {
                let integer = *registers.integers.get(return_address.index);

                thread.r#return = Some(Some(ConcreteValue::Integer(integer)));
            }
            _ => todo!(),
        };
    } else {
        thread.r#return = Some(None);
    }
}
