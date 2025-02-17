use std::{rc::Rc, thread::JoinHandle};

use tracing::info;

use crate::{
    instruction::TypeCode, vm::CallFrame, Chunk, ConcreteValue, DustString, Operation, Span, Value,
};

pub struct Thread {
    chunk: Rc<Chunk>,
    call_stack: Vec<CallFrame>,
    _spawned_threads: Vec<JoinHandle<()>>,
}

impl Thread {
    pub fn new(chunk: Rc<Chunk>) -> Self {
        let mut call_stack = Vec::with_capacity(chunk.prototypes.len() + 1);
        let main_call = CallFrame::new(Rc::clone(&chunk), 0);

        call_stack.push(main_call);

        Thread {
            chunk,
            call_stack,
            _spawned_threads: Vec::new(),
        }
    }

    pub fn run(mut self) -> Option<Value> {
        info!(
            "Starting thread {}",
            self.chunk
                .name
                .clone()
                .unwrap_or_else(|| DustString::from("anonymous"))
        );

        while !self.call_stack.is_empty() {
            let current_frame = self.current_frame_mut();
            let instructions = &current_frame.chunk.instructions;
            let ip = {
                let current = current_frame.ip;
                current_frame.ip += 1;

                current
            };

            assert!(ip < instructions.len(), "IP out of bounds");

            let instruction = &instructions[ip];

            info!("Run instruction {}", instruction.operation());

            match instruction.operation() {
                Operation::LOAD_CONSTANT => {
                    let destination = instruction.a_field() as usize;
                    let constant_index = instruction.b_field() as usize;
                    let constant_type = instruction.b_type();
                    let jump_next = instruction.c_field() != 0;

                    match constant_type {
                        TypeCode::CHARACTER => {
                            let character = current_frame.get_character_constant(constant_index);

                            current_frame
                                .registers
                                .characters
                                .get_mut(destination)
                                .set(character);
                        }
                        TypeCode::FLOAT => {
                            let float = current_frame.get_float_constant(constant_index);

                            current_frame
                                .registers
                                .floats
                                .get_mut(destination)
                                .set(float);
                        }
                        TypeCode::INTEGER => {
                            let integer = current_frame.get_integer_constant(constant_index);

                            current_frame
                                .registers
                                .integers
                                .get_mut(destination)
                                .set(integer);
                        }
                        TypeCode::STRING => {
                            let string = current_frame.get_string_constant(constant_index).clone();

                            current_frame
                                .registers
                                .strings
                                .get_mut(destination)
                                .set(string);
                        }
                        _ => unreachable!(),
                    }

                    if jump_next {
                        current_frame.ip += 1;
                    }
                }
                Operation::LESS => match (instruction.b_type(), instruction.c_type()) {
                    (TypeCode::INTEGER, TypeCode::INTEGER) => {
                        let left = instruction.b_field() as usize;
                        let left_is_constant = instruction.b_is_constant();
                        let right = instruction.c_field() as usize;
                        let right_is_constant = instruction.c_is_constant();
                        let comparator = instruction.d_field();

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
                            current_frame.ip += 1;
                        }
                    }
                    _ => todo!(),
                },
                Operation::ADD => match (instruction.b_type(), instruction.c_type()) {
                    (TypeCode::BYTE, TypeCode::BYTE) => {
                        let left_index = instruction.b_field() as usize;
                        let right_index = instruction.c_field() as usize;
                        let destination_index = instruction.a_field() as usize;

                        let left_value = current_frame.get_byte_from_register(left_index);
                        let right_value = current_frame.get_byte_from_register(right_index);
                        let sum = left_value + right_value;

                        current_frame
                            .registers
                            .bytes
                            .set_to_new_register(destination_index, sum);
                    }
                    (TypeCode::CHARACTER, TypeCode::CHARACTER) => {
                        let left_index = instruction.b_field() as usize;
                        let left_is_constant = instruction.b_is_constant();
                        let right_index = instruction.c_field() as usize;
                        let right_is_constant = instruction.c_is_constant();
                        let destination_index = instruction.a_field() as usize;

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
                        let concatenated = {
                            let mut concatenated = DustString::from(String::with_capacity(2));

                            concatenated.push(left_value);
                            concatenated.push(right_value);

                            concatenated
                        };

                        current_frame
                            .registers
                            .strings
                            .set_to_new_register(destination_index, concatenated);
                    }
                    (TypeCode::CHARACTER, TypeCode::STRING) => {
                        let left_index = instruction.b_field() as usize;
                        let left_is_constant = instruction.b_is_constant();
                        let right_index = instruction.c_field() as usize;
                        let right_is_constant = instruction.c_is_constant();
                        let destination_index = instruction.a_field() as usize;

                        let left_value = if left_is_constant {
                            current_frame.get_character_constant(left_index)
                        } else {
                            current_frame.get_character_from_register(left_index)
                        };
                        let right_value = if right_is_constant {
                            current_frame.get_string_constant(right_index)
                        } else {
                            current_frame.get_string_from_register(right_index)
                        };
                        let concatenated = DustString::from(format!("{left_value}{right_value}"));

                        current_frame
                            .registers
                            .strings
                            .set_to_new_register(destination_index, concatenated);
                    }
                    (TypeCode::FLOAT, TypeCode::FLOAT) => {
                        let left_index = instruction.b_field() as usize;
                        let right_index = instruction.c_field() as usize;
                        let destination_index = instruction.a_field() as usize;

                        let left_value = current_frame.get_float_from_register(left_index);
                        let right_value = current_frame.get_float_from_register(right_index);
                        let sum = left_value + right_value;

                        current_frame
                            .registers
                            .floats
                            .set_to_new_register(destination_index, sum);
                    }
                    (TypeCode::INTEGER, TypeCode::INTEGER) => {
                        let left = instruction.b_field() as usize;
                        let left_is_constant = instruction.b_is_constant();
                        let right = instruction.c_field() as usize;
                        let right_is_constant = instruction.c_is_constant();
                        let destination_index = instruction.a_field() as usize;

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
                        let sum = left_value + right_value;

                        current_frame
                            .registers
                            .integers
                            .set_to_new_register(destination_index, sum);
                    }
                    (TypeCode::STRING, TypeCode::STRING) => {
                        let left = instruction.b_field() as usize;
                        let left_is_constant = instruction.b_is_constant();
                        let right = instruction.c_field() as usize;
                        let right_is_constant = instruction.c_is_constant();
                        let destination_index = instruction.a_field() as usize;

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
                            .set_to_new_register(destination_index, concatenated);
                    }
                    _ => todo!(),
                },
                Operation::JUMP => {
                    let offset = instruction.b_field() as usize;
                    let is_positive = instruction.c_field() != 0;

                    if is_positive {
                        current_frame.ip += offset;
                    } else {
                        current_frame.ip -= offset + 1;
                    }
                }
                Operation::RETURN => {
                    let should_return_value = instruction.b_field() != 0;
                    let return_register = instruction.c_field() as usize;
                    let return_type = instruction.b_type();

                    if should_return_value {
                        match return_type {
                            TypeCode::BOOLEAN => {
                                let return_value =
                                    current_frame.get_boolean_from_register(return_register);

                                return Some(Value::boolean(return_value));
                            }
                            TypeCode::BYTE => {
                                let return_value =
                                    current_frame.get_byte_from_register(return_register);

                                return Some(Value::byte(return_value));
                            }
                            TypeCode::CHARACTER => {
                                let return_value =
                                    current_frame.get_character_from_register(return_register);

                                return Some(Value::character(return_value));
                            }
                            TypeCode::FLOAT => {
                                let return_value =
                                    current_frame.get_float_from_register(return_register);

                                return Some(Value::float(return_value));
                            }
                            TypeCode::INTEGER => {
                                let return_value =
                                    current_frame.get_integer_from_register(return_register);

                                return Some(Value::integer(return_value));
                            }
                            TypeCode::STRING => {
                                let return_value = current_frame
                                    .get_string_from_register(return_register)
                                    .clone();

                                return Some(Value::string(return_value));
                            }
                            TypeCode::LIST => {
                                let abstract_list =
                                    current_frame.get_list_from_register(return_register);

                                let mut concrete_list =
                                    Vec::with_capacity(abstract_list.item_pointers.len());

                                match abstract_list.item_type {
                                    TypeCode::BOOLEAN => {
                                        for pointer in &abstract_list.item_pointers {
                                            let boolean =
                                                current_frame.get_boolean_from_pointer(&pointer);
                                            let value = ConcreteValue::Boolean(boolean);

                                            concrete_list.push(value);
                                        }
                                    }
                                    TypeCode::BYTE => {
                                        for pointer in &abstract_list.item_pointers {
                                            let byte =
                                                current_frame.get_byte_from_pointer(&pointer);
                                            let value = ConcreteValue::Byte(byte);

                                            concrete_list.push(value);
                                        }
                                    }
                                    TypeCode::CHARACTER => {
                                        for pointer in &abstract_list.item_pointers {
                                            let character =
                                                current_frame.get_character_from_pointer(&pointer);
                                            let value = ConcreteValue::Character(character);

                                            concrete_list.push(value);
                                        }
                                    }
                                    TypeCode::FLOAT => {
                                        for pointer in &abstract_list.item_pointers {
                                            let float =
                                                current_frame.get_float_from_pointer(&pointer);
                                            let value = ConcreteValue::Float(float);

                                            concrete_list.push(value);
                                        }
                                    }
                                    TypeCode::INTEGER => {
                                        for pointer in &abstract_list.item_pointers {
                                            let integer =
                                                current_frame.get_integer_from_pointer(&pointer);
                                            let value = ConcreteValue::Integer(integer);

                                            concrete_list.push(value);
                                        }
                                    }
                                    TypeCode::STRING => {
                                        for pointer in &abstract_list.item_pointers {
                                            let string = current_frame
                                                .get_string_from_pointer(pointer)
                                                .clone();
                                            let value = ConcreteValue::String(string);

                                            concrete_list.push(value);
                                        }
                                    }
                                    _ => todo!(),
                                }

                                return Some(Value::Concrete(ConcreteValue::list(
                                    concrete_list,
                                    abstract_list.item_type,
                                )));
                            }
                            _ => unreachable!(),
                        }
                    } else {
                        return None;
                    }
                }
                unimplemented => {
                    todo!("{unimplemented} has not been implemented in the VM");
                }
            }
        }

        None
    }

    pub fn current_position(&self) -> Span {
        let current_frame = self.current_frame();

        current_frame.chunk.positions[current_frame.ip]
    }

    pub fn current_frame(&self) -> &CallFrame {
        if cfg!(debug_assertions) {
            self.call_stack.last().unwrap()
        } else {
            unsafe { self.call_stack.last().unwrap_unchecked() }
        }
    }

    pub fn current_frame_mut(&mut self) -> &mut CallFrame {
        if cfg!(debug_assertions) {
            self.call_stack.last_mut().unwrap()
        } else {
            unsafe { self.call_stack.last_mut().unwrap_unchecked() }
        }
    }
}
