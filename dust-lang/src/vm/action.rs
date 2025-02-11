use std::{
    arch::asm,
    fmt::{self, Display, Formatter},
};

use crate::{
    AbstractList, ConcreteValue, DustString, Instruction, Operation, Value,
    instruction::{InstructionFields, Jump, TypeCode},
};

use super::{Pointer, Register, call_frame::PointerCache, thread::Thread};

#[derive(Debug)]
pub struct ActionSequence {
    pub actions: Vec<Action>,
}

impl ActionSequence {
    #[allow(clippy::while_let_on_iterator)]
    pub fn new(instructions: &[Instruction]) -> Self {
        let mut instructions = instructions.iter().rev();
        let mut actions = Vec::with_capacity(instructions.len());

        while let Some(instruction) = instructions.next() {
            if instruction.operation() == Operation::JUMP {
                let Jump {
                    offset: backward_offset,
                    is_positive,
                } = Jump::from(instruction);

                if !is_positive {
                    let mut previous = instruction;

                    actions.push(Action::optimized(instruction));

                    while let Some(instruction) = instructions.next() {
                        actions.push(Action::optimized(instruction));

                        if matches!(
                            instruction.operation(),
                            Operation::EQUAL | Operation::LESS | Operation::LESS_EQUAL
                        ) && previous.operation() == Operation::JUMP
                        {
                            let Jump {
                                offset: forward_offset,
                                is_positive,
                            } = Jump::from(previous);

                            if is_positive && forward_offset == backward_offset - 1 {
                                break;
                            }
                        }

                        previous = instruction;
                    }

                    continue;
                }
            }

            let action = Action::unoptimized(instruction);

            actions.push(action);
        }

        actions.reverse();

        ActionSequence { actions }
    }

    pub fn len(&self) -> usize {
        self.actions.len()
    }
}

impl Display for ActionSequence {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "[")?;

        for (index, action) in self.actions.iter().enumerate() {
            write!(f, "{}", action.name)?;

            if index < self.actions.len() - 1 {
                write!(f, ", ")?;
            }
        }

        write!(f, "]")
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Action {
    pub name: &'static str,
    pub logic: ActionLogic,
    pub instruction: InstructionFields,
}

impl Action {
    fn optimized(instruction: &Instruction) -> Self {
        let instruction = InstructionFields::from(instruction);
        let (name, logic): (&'static str, ActionLogic) = match (
            instruction.operation,
            instruction.b_type,
            instruction.c_type,
        ) {
            (Operation::ADD, TypeCode::INTEGER, TypeCode::INTEGER) => {
                ("ADD_INTEGERS", add_integers_optimized)
            }
            (Operation::LESS, TypeCode::INTEGER, TypeCode::INTEGER) => {
                ("LESS_INTEGERS", less_integers_optimized)
            }
            (Operation::JUMP, _, _) => ("JUMP", jump),
            _ => todo!(),
        };

        Action {
            name,
            logic,
            instruction,
        }
    }

    fn unoptimized(instruction: &Instruction) -> Self {
        let instruction = InstructionFields::from(instruction);
        let logic = match instruction.operation {
            Operation::POINT => point,
            Operation::CLOSE => close,
            Operation::LOAD_ENCODED => load_encoded,
            Operation::LOAD_CONSTANT => load_constant,
            Operation::LOAD_LIST => load_list,
            Operation::LOAD_FUNCTION => load_function,
            Operation::LOAD_SELF => load_self,
            Operation::ADD => add,
            Operation::SUBTRACT => subtract,
            Operation::MULTIPLY => multiply,
            Operation::DIVIDE => divide,
            Operation::MODULO => modulo,
            Operation::EQUAL => equal,
            Operation::LESS => less,
            Operation::LESS_EQUAL => less_equal,
            Operation::TEST => test,
            Operation::TEST_SET => test_set,
            Operation::RETURN => r#return,
            _ => todo!(),
        };

        Action {
            name: instruction.operation.name(),
            logic,
            instruction,
        }
    }
}

pub type ActionLogic = fn(InstructionFields, &mut Thread);

pub fn point(instruction: InstructionFields, thread: &mut Thread) {
    let destination = instruction.a_field as usize;
    let to = instruction.b_field as usize;
    let to_is_constant = instruction.b_is_constant;
    let r#type = instruction.b_type;

    match r#type {
        TypeCode::BOOLEAN => {
            let boolean = if to_is_constant {
                if cfg!(debug_assertions) {
                    thread.get_constant(to).as_boolean().unwrap()
                } else {
                    unsafe { thread.get_constant(to).as_boolean().unwrap_unchecked() }
                }
            } else {
                thread.get_boolean_register(to)
            };
            let register = Register::Value(*boolean);

            thread.set_boolean_register(destination, register);
        }
        TypeCode::BYTE => {
            let byte = if to_is_constant {
                if cfg!(debug_assertions) {
                    thread.get_constant(to).as_byte().unwrap()
                } else {
                    unsafe { thread.get_constant(to).as_byte().unwrap_unchecked() }
                }
            } else {
                thread.get_byte_register(to)
            };
            let register = Register::Value(*byte);

            thread.set_byte_register(destination, register);
        }
        TypeCode::CHARACTER => {
            let character = if to_is_constant {
                if cfg!(debug_assertions) {
                    thread.get_constant(to).as_character().unwrap()
                } else {
                    unsafe { thread.get_constant(to).as_character().unwrap_unchecked() }
                }
            } else {
                thread.get_character_register(to)
            };
            let register = Register::Value(*character);

            thread.set_character_register(destination, register);
        }
        TypeCode::FLOAT => {
            let float = if to_is_constant {
                if cfg!(debug_assertions) {
                    thread.get_constant(to).as_float().unwrap()
                } else {
                    unsafe { thread.get_constant(to).as_float().unwrap_unchecked() }
                }
            } else {
                thread.get_float_register(to)
            };
            let register = Register::Value(*float);

            thread.set_float_register(destination, register);
        }
        TypeCode::INTEGER => {
            let integer = if to_is_constant {
                if cfg!(debug_assertions) {
                    thread.get_constant(to).as_integer().unwrap()
                } else {
                    unsafe { thread.get_constant(to).as_integer().unwrap_unchecked() }
                }
            } else {
                thread.get_integer_register(to)
            };
            let register = Register::Value(*integer);

            thread.set_integer_register(destination, register);
        }
        TypeCode::STRING => {
            let string = if to_is_constant {
                if cfg!(debug_assertions) {
                    thread.get_constant(to).as_string().unwrap().clone()
                } else {
                    unsafe {
                        thread
                            .get_constant(to)
                            .as_string()
                            .unwrap_unchecked()
                            .clone()
                    }
                }
            } else {
                thread.get_string_register(to).clone()
            };
            let register = Register::Value(string);

            thread.set_string_register(destination, register);
        }
        TypeCode::LIST => {
            let list = thread.get_list_register(to).clone();
            let register = Register::Value(list);

            thread.set_list_register(destination, register);
        }
        _ => unreachable!(),
    }
}

pub fn close(instruction: InstructionFields, thread: &mut Thread) {
    let from = instruction.b_field as usize;
    let to = instruction.c_field as usize;
    let r#type = instruction.b_type;

    match r#type {
        TypeCode::BOOLEAN => {
            for register_index in from..=to {
                thread.close_boolean_register(register_index);
            }
        }
        TypeCode::BYTE => {
            for register_index in from..=to {
                thread.close_byte_register(register_index);
            }
        }
        TypeCode::CHARACTER => {
            for register_index in from..=to {
                thread.close_character_register(register_index);
            }
        }
        TypeCode::FLOAT => {
            for register_index in from..=to {
                thread.close_float_register(register_index);
            }
        }
        TypeCode::INTEGER => {
            for register_index in from..=to {
                thread.close_integer_register(register_index);
            }
        }
        TypeCode::STRING => {
            for register_index in from..=to {
                thread.close_string_register(register_index);
            }
        }
        TypeCode::LIST => {
            for register_index in from..=to {
                thread.close_list_register(register_index);
            }
        }
        _ => unreachable!(),
    }
}

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
        _ => unreachable!(),
    }

    if jump_next {
        thread.current_frame_mut().ip += 1;
    }
}

pub fn load_list(instruction: InstructionFields, thread: &mut Thread) {
    let destination = instruction.a_field;
    let start_register = instruction.b_field;
    let item_type = instruction.b_type;
    let end_register = instruction.c_field;
    let jump_next = instruction.d_field;

    let length = (end_register - start_register + 1) as usize;
    let mut item_pointers = Vec::with_capacity(length);

    for register_index in start_register..=end_register {
        let register_index = register_index as usize;

        let pointer = match item_type {
            TypeCode::BOOLEAN => {
                let is_closed = thread.is_boolean_register_closed(register_index);

                if is_closed {
                    continue;
                }

                Pointer::RegisterBoolean(register_index)
            }
            TypeCode::BYTE => {
                let is_closed = thread.is_byte_register_closed(register_index);

                if is_closed {
                    continue;
                }

                Pointer::RegisterByte(register_index)
            }
            TypeCode::CHARACTER => {
                let is_closed = thread.is_character_register_closed(register_index);

                if is_closed {
                    continue;
                }

                Pointer::RegisterCharacter(register_index)
            }
            TypeCode::FLOAT => {
                let is_closed = thread.is_float_register_closed(register_index);

                if is_closed {
                    continue;
                }

                Pointer::RegisterFloat(register_index)
            }
            TypeCode::INTEGER => {
                let is_closed = thread.is_integer_register_closed(register_index);

                if is_closed {
                    continue;
                }

                Pointer::RegisterInteger(register_index)
            }
            TypeCode::STRING => {
                let is_closed = thread.is_string_register_closed(register_index);

                if is_closed {
                    continue;
                }

                Pointer::RegisterString(register_index)
            }
            TypeCode::LIST => {
                let is_closed = thread.is_list_register_closed(register_index);

                if is_closed {
                    continue;
                }

                Pointer::RegisterList(register_index)
            }
            _ => unreachable!(),
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

pub fn load_function(_: InstructionFields, _: &mut Thread) {
    todo!()
}

pub fn load_self(_: InstructionFields, _: &mut Thread) {
    todo!()
}

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

fn add_integers_optimized(instruction: InstructionFields, thread: &mut Thread) {
    let current_frame = thread.current_frame_mut();
    let pointer_cache = current_frame.pointer_caches[current_frame.ip];

    if let PointerCache::Integers { left, right } = pointer_cache {
        let destination = instruction.a_field as usize;
        let destination_pointer = thread.get_integer_register_mut(destination) as *mut i64;

        unsafe {
            asm!(
                "add {}, {}",
                inout(reg) *left => *destination_pointer,
                in(reg) *right,
            )
        }
    } else if let PointerCache::Empty = pointer_cache {
        let left = instruction.b_field as usize;
        let right = instruction.c_field as usize;
        let left_is_constant = instruction.b_is_constant;
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
        let cache = PointerCache::Integers {
            left: left_value,
            right: right_value,
        };
        let destination = instruction.a_field as usize;
        let sum = left_value.saturating_add(*right_value);
        let register = Register::Value(sum);

        thread.set_integer_register(destination, register);
        thread.set_current_pointer_cache(cache);
    } else {
        unreachable!()
    }
}

pub fn subtract(instruction: InstructionFields, thread: &mut Thread) {
    let destination = instruction.a_field as usize;
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
            let result = left_value.saturating_sub(*right_value);
            let register = Register::Value(result);

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
            let result = left_value.saturating_sub(*right_value);
            let register = Register::Value(result);

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
            let result = left_value - right_value;
            let register = Register::Value(result);

            thread.set_float_register(destination, register);
        }
        _ => unreachable!(),
    }
}

pub fn multiply(instruction: InstructionFields, thread: &mut Thread) {
    let destination = instruction.a_field as usize;
    let left = instruction.b_field as usize;
    let left_type = instruction.b_type;
    let left_is_constant = instruction.b_is_constant;
    let right = instruction.c_field as usize;
    let right_type = instruction.c_type;
    let right_is_constant = instruction.c_is_constant;

    match (left_type, right_type) {
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
            let result = left_value.saturating_mul(*right_value);
            let register = Register::Value(result);

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
            let result = left_value * right_value;
            let register = Register::Value(result);

            thread.set_float_register(destination, register);
        }
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
            let result = left_value.saturating_mul(*right_value);
            let register = Register::Value(result);

            thread.set_integer_register(destination, register);
        }
        _ => unreachable!(),
    }
}

pub fn divide(instruction: InstructionFields, thread: &mut Thread) {
    let destination = instruction.a_field as usize;
    let left = instruction.b_field as usize;
    let left_type = instruction.b_type;
    let left_is_constant = instruction.b_is_constant;
    let right = instruction.c_field as usize;
    let right_type = instruction.c_type;
    let right_is_constant = instruction.c_is_constant;

    match (left_type, right_type) {
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
            let result = left_value.saturating_div(*right_value);
            let register = Register::Value(result);

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
            let result = left_value / right_value;
            let register = Register::Value(result);

            thread.set_float_register(destination, register);
        }
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
            let result = left_value.saturating_div(*right_value);
            let register = Register::Value(result);

            thread.set_integer_register(destination, register);
        }
        _ => unreachable!(),
    }
}

pub fn modulo(instruction: InstructionFields, thread: &mut Thread) {
    let destination = instruction.a_field as usize;
    let left = instruction.b_field as usize;
    let left_type = instruction.b_type;
    let left_is_constant = instruction.b_is_constant;
    let right = instruction.c_field as usize;
    let right_type = instruction.c_type;
    let right_is_constant = instruction.c_is_constant;

    match (left_type, right_type) {
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
            let result = left_value % right_value;
            let register = Register::Value(result);

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
            let result = left_value % right_value;
            let register = Register::Value(result);

            thread.set_float_register(destination, register);
        }
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
            let result = left_value % right_value;
            let register = Register::Value(result);

            thread.set_integer_register(destination, register);
        }
        _ => unreachable!(),
    }
}

pub fn test(instruction: InstructionFields, thread: &mut Thread) {
    let operand_register = instruction.b_field as usize;
    let test_value = instruction.c_field != 0;
    let operand_boolean = thread.get_boolean_register(operand_register);

    if *operand_boolean == test_value {
        thread.current_frame_mut().ip += 1;
    }
}

pub fn test_set(_: InstructionFields, _: &mut Thread) {
    todo!()
}

pub fn equal(instruction: InstructionFields, thread: &mut Thread) {
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
            let result = left_value == right_value;

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
            let result = left_value == right_value;

            if result == comparator {
                thread.current_frame_mut().ip += 1;
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
            let result = left_value == right_value;

            if result == comparator {
                thread.current_frame_mut().ip += 1;
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
            let result = left_value == right_value;

            if result == comparator {
                thread.current_frame_mut().ip += 1;
            }
        }
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
            let result = left_value == right_value;

            if result == comparator {
                thread.current_frame_mut().ip += 1;
            }
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
            let result = left_value == right_value;

            if result == comparator {
                thread.current_frame_mut().ip += 1;
            }
        }
        _ => unreachable!(),
    }
}

pub fn less(instruction: InstructionFields, thread: &mut Thread) {
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
                thread.current_frame_mut().ip += 1;
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
                thread.current_frame_mut().ip += 1;
            }
        }
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
                thread.current_frame_mut().ip += 1;
            }
        }
        _ => unreachable!(),
    }
}

fn less_integers_optimized(instruction: InstructionFields, thread: &mut Thread) {
    let current_frame = thread.current_frame_mut();
    let pointer_cache = current_frame.pointer_caches[current_frame.ip];

    let is_less = if let PointerCache::Integers { left, right } = pointer_cache {
        let left_value = unsafe { *left };
        let right_value = unsafe { *right };

        left_value < right_value
    } else if let PointerCache::Empty = pointer_cache {
        let left = instruction.b_field as usize;
        let right = instruction.c_field as usize;
        let left_is_constant = instruction.b_is_constant;
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
        let cache = PointerCache::Integers {
            left: left_value,
            right: right_value,
        };
        let is_less = left_value < right_value;

        thread.set_current_pointer_cache(cache);

        is_less
    } else {
        unreachable!()
    };

    if is_less {
        thread.current_frame_mut().ip += 1;
    }
}

pub fn less_equal(instruction: InstructionFields, thread: &mut Thread) {
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
            let result = left_value <= right_value;

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
            let result = left_value <= right_value;

            if result == comparator {
                thread.current_frame_mut().ip += 1;
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
            let result = left_value <= right_value;

            if result == comparator {
                thread.current_frame_mut().ip += 1;
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
            let result = left_value <= right_value;

            if result == comparator {
                thread.current_frame_mut().ip += 1;
            }
        }
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
            let result = left_value <= right_value;

            if result == comparator {
                thread.current_frame_mut().ip += 1;
            }
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
            let result = left_value <= right_value;

            if result == comparator {
                thread.current_frame_mut().ip += 1;
            }
        }
        _ => unreachable!(),
    }
}

pub fn negate(_: InstructionFields, _: &mut Thread) {
    todo!()
}

pub fn not(_: InstructionFields, _: &mut Thread) {
    todo!()
}

pub fn jump(instruction: InstructionFields, thread: &mut Thread) {
    let offset = instruction.b_field as usize;
    let is_positive = instruction.c_field != 0;

    if is_positive {
        thread.current_frame_mut().ip += offset;
    } else {
        thread.current_frame_mut().ip -= offset + 1;
    }
}

pub fn call(_: InstructionFields, _: &mut Thread) {
    todo!()
}

pub fn call_native(_: InstructionFields, _: &mut Thread) {
    todo!()
}

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
                let mut items = Vec::with_capacity(abstract_list.item_pointers.len());

                for pointer in &abstract_list.item_pointers {
                    let value = thread.get_value_from_pointer(pointer);

                    items.push(value);
                }

                thread.return_value = Some(Some(Value::Concrete(ConcreteValue::List {
                    items,
                    item_type: abstract_list.item_type,
                })));
            }
            _ => unreachable!(),
        }
    } else {
        thread.return_value = Some(None);
    }
}
