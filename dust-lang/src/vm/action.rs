use crate::{
    AbstractList, ConcreteValue, Instruction, Value,
    instruction::{InstructionFields, TypeCode},
};

use super::{Pointer, Register, thread::Thread};

#[derive(Debug)]
pub struct ActionSequence {
    pub actions: Vec<Action>,
}

impl ActionSequence {
    pub fn new(instructions: &[Instruction]) -> Self {
        let mut actions = Vec::with_capacity(instructions.len());

        for instruction in instructions {
            let action = Action::from(instruction);

            actions.push(action);
        }

        ActionSequence { actions }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Action {
    pub logic: RunnerLogic,
    pub instruction: InstructionFields,
}

impl From<&Instruction> for Action {
    fn from(instruction: &Instruction) -> Self {
        let operation = instruction.operation();
        let logic = RUNNER_LOGIC_TABLE[operation.0 as usize];
        let instruction = InstructionFields::from(instruction);

        Action { logic, instruction }
    }
}

pub type RunnerLogic = fn(InstructionFields, &mut Thread);

pub const RUNNER_LOGIC_TABLE: [RunnerLogic; 23] = [
    point,
    close,
    load_encoded,
    load_constant,
    load_function,
    load_list,
    load_self,
    add,
    subtract,
    multiply,
    divide,
    modulo,
    equal,
    less,
    less_equal,
    negate,
    not,
    test,
    test_set,
    call,
    call_native,
    jump,
    r#return,
];

pub fn point(_: InstructionFields, thread: &mut Thread) {}

pub fn close(_: InstructionFields, thread: &mut Thread) {}

pub fn load_encoded(instruction: InstructionFields, thread: &mut Thread) {
    let destination = instruction.a_field;
    let value = instruction.b_field;
    let value_type = instruction.b_type;
    let jump_next = instruction.c_field != 0;

    match value_type {
        TypeCode::BOOLEAN => {
            let register = Register::Value(value != 0);

            thread.set_boolean_register(destination as usize, register);
        }
        TypeCode::BYTE => {
            let register = Register::Value(value as u8);

            thread.set_byte_register(destination as usize, register);
        }
        _ => unreachable!(),
    }

    if jump_next {
        thread.current_frame_mut().ip += 1;
    }
}

pub fn load_constant(instruction: InstructionFields, thread: &mut Thread) {
    let destination = instruction.a_field as usize;
    let constant_index = instruction.b_field as usize;
    let constant_type = instruction.b_type;
    let jump_next = instruction.c_field != 0;

    match constant_type {
        TypeCode::CHARACTER => {
            let constant = *thread.get_constant(constant_index).as_character().unwrap();
            let register = Register::Value(constant);

            thread.set_character_register(destination, register);
        }
        TypeCode::FLOAT => {
            let constant = *thread.get_constant(constant_index).as_float().unwrap();
            let register = Register::Value(constant);

            thread.set_float_register(destination, register);
        }
        TypeCode::INTEGER => {
            let constant = *thread.get_constant(constant_index).as_integer().unwrap();
            let register = Register::Value(constant);

            thread.set_integer_register(destination, register);
        }
        TypeCode::STRING => {
            let register = Register::Pointer(Pointer::ConstantString(constant_index));

            thread.set_string_register(destination, register);
        }
        _ => unimplemented!(),
    }

    if jump_next {
        thread.current_frame_mut().ip += 1;
    }
}

pub fn load_list(instruction: InstructionFields, thread: &mut Thread) {
    let destination = instruction.a_field;
    let start_register = instruction.b_field;
    let item_type = instruction.b_type;
    let length = instruction.c_field;
    let jump_next = instruction.d_field;

    let mut item_pointers = Vec::with_capacity(length as usize);

    for register_index in start_register..start_register + length {
        let pointer = match item_type {
            TypeCode::BOOLEAN => Pointer::RegisterBoolean(register_index as usize),
            TypeCode::BYTE => Pointer::RegisterByte(register_index as usize),
            TypeCode::CHARACTER => Pointer::RegisterCharacter(register_index as usize),
            TypeCode::FLOAT => Pointer::RegisterFloat(register_index as usize),
            TypeCode::INTEGER => Pointer::RegisterInteger(register_index as usize),
            TypeCode::STRING => Pointer::RegisterString(register_index as usize),
            TypeCode::LIST => Pointer::RegisterList(register_index as usize),
            _ => unimplemented!(),
        };

        item_pointers.push(pointer);
    }

    let abstract_list = AbstractList {
        item_type,
        item_pointers,
    };
    let register = Register::Value(abstract_list);

    thread.set_list_register(destination as usize, register);

    if jump_next {
        thread.current_frame_mut().ip += 1;
    }
}

pub fn load_function(instruction: InstructionFields, thread: &mut Thread) {}

pub fn load_self(instruction: InstructionFields, thread: &mut Thread) {}

pub fn add(instruction: InstructionFields, thread: &mut Thread) {
    let destination = instruction.a_field as usize;
    let left = instruction.b_field as usize;
    let left_is_constant = instruction.b_is_constant;
    let left_type = instruction.b_type;
    let right = instruction.c_field as usize;
    let right_is_constant = instruction.c_is_constant;
    let right_type = instruction.c_type;

    match (left_type, right_type) {
        (TypeCode::INTEGER, TypeCode::INTEGER) => {
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
            let sum = left_value + right_value;
            let register = Register::Value(sum);

            thread.set_integer_register(destination, register);
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
            let sum = left_value + right_value;
            let register = Register::Value(sum);

            thread.set_byte_register(destination, register);
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
        _ => unimplemented!(),
    }
}

pub fn subtract(instruction: InstructionFields, thread: &mut Thread) {}

pub fn multiply(instruction: InstructionFields, thread: &mut Thread) {}

pub fn divide(instruction: InstructionFields, thread: &mut Thread) {}

pub fn modulo(instruction: InstructionFields, thread: &mut Thread) {}

pub fn test(instruction: InstructionFields, thread: &mut Thread) {}

pub fn test_set(instruction: InstructionFields, thread: &mut Thread) {}

pub fn equal(instruction: InstructionFields, thread: &mut Thread) {}

pub fn less(instruction: InstructionFields, thread: &mut Thread) {
    let comparator = instruction.d_field;
    let left = instruction.b_field as usize;
    let left_type = instruction.b_type;
    let left_is_constant = instruction.b_is_constant;
    let right = instruction.c_field as usize;
    let right_type = instruction.c_type;
    let right_is_constant = instruction.c_is_constant;

    match (left_type, right_type) {
        (TypeCode::INTEGER, TypeCode::INTEGER) => {
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
            let result = left_value < right_value;

            if result == comparator {
                thread.current_frame_mut().ip += 1;
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
                thread.current_frame_mut().ip += 1;
            }
        }
        _ => unimplemented!(),
    }
}

pub fn less_equal(instruction: InstructionFields, thread: &mut Thread) {}

pub fn negate(instruction: InstructionFields, thread: &mut Thread) {}

pub fn not(instruction: InstructionFields, thread: &mut Thread) {}

pub fn jump(instruction: InstructionFields, thread: &mut Thread) {
    let offset = instruction.b_field as usize;
    let is_positive = instruction.c_field != 0;

    if is_positive {
        thread.current_frame_mut().ip += offset;
    } else {
        thread.current_frame_mut().ip -= offset + 1;
    }
}

pub fn call(instruction: InstructionFields, thread: &mut Thread) {}

pub fn call_native(instruction: InstructionFields, thread: &mut Thread) {}

pub fn r#return(instruction: InstructionFields, thread: &mut Thread) {
    let should_return_value = instruction.b_field != 0;
    let return_register = instruction.c_field as usize;
    let return_type = instruction.b_type;

    if should_return_value {
        match return_type {
            TypeCode::BOOLEAN => {
                let return_value = *thread.get_boolean_register(return_register);
                thread.return_value = Some(Some(Value::boolean(return_value)));
            }
            TypeCode::BYTE => {
                let return_value = *thread.get_byte_register(return_register);
                thread.return_value = Some(Some(Value::byte(return_value)));
            }
            TypeCode::CHARACTER => {
                let return_value = *thread.get_character_register(return_register);
                thread.return_value = Some(Some(Value::character(return_value)));
            }
            TypeCode::FLOAT => {
                let return_value = *thread.get_float_register(return_register);
                thread.return_value = Some(Some(Value::float(return_value)));
            }
            TypeCode::INTEGER => {
                let return_value = *thread.get_integer_register(return_register);
                thread.return_value = Some(Some(Value::integer(return_value)));
            }
            TypeCode::STRING => {
                let return_value = thread.get_string_register(return_register).clone();
                thread.return_value = Some(Some(Value::string(return_value)));
            }
            TypeCode::LIST => {
                let abstract_list = thread.get_list_register(return_register).clone();
                let mut concrete_list = Vec::with_capacity(abstract_list.item_pointers.len());

                for pointer in &abstract_list.item_pointers {
                    concrete_list.push(thread.get_value_from_pointer(pointer));
                }

                thread.return_value =
                    Some(Some(Value::Concrete(ConcreteValue::List(concrete_list))));
            }
            _ => unimplemented!(),
        }
    } else {
        thread.return_value = Some(None);
    }
}

#[cfg(test)]
mod tests {

    use crate::Operation;

    use super::*;

    const ALL_OPERATIONS: [(Operation, RunnerLogic); 23] = [
        (Operation::POINT, point),
        (Operation::CLOSE, close),
        (Operation::LOAD_ENCODED, load_encoded),
        (Operation::LOAD_CONSTANT, load_constant),
        (Operation::LOAD_FUNCTION, load_function),
        (Operation::LOAD_LIST, load_list),
        (Operation::LOAD_SELF, load_self),
        (Operation::ADD, add),
        (Operation::SUBTRACT, subtract),
        (Operation::MULTIPLY, multiply),
        (Operation::DIVIDE, divide),
        (Operation::MODULO, modulo),
        (Operation::TEST, test),
        (Operation::TEST_SET, test_set),
        (Operation::EQUAL, equal),
        (Operation::LESS, less),
        (Operation::LESS_EQUAL, less_equal),
        (Operation::NEGATE, negate),
        (Operation::NOT, not),
        (Operation::CALL, call),
        (Operation::CALL_NATIVE, call_native),
        (Operation::JUMP, jump),
        (Operation::RETURN, r#return),
    ];

    #[test]
    fn operations_map_to_the_correct_runner() {
        for (operation, expected_runner) in ALL_OPERATIONS {
            let actual_runner = RUNNER_LOGIC_TABLE[operation.0 as usize];

            assert_eq!(
                expected_runner, actual_runner,
                "{operation} runner is incorrect"
            );
        }
    }
}
