use std::{sync::Arc, thread::JoinHandle};

use tracing::info;

use crate::{
    instruction::TypeCode,
    vm::{CallFrame, Pointer},
    AbstractList, Chunk, DustString, Operation, Span, Value,
};

pub struct Thread {
    chunk: Arc<Chunk>,
    call_stack: Vec<CallFrame>,
    _spawned_threads: Vec<JoinHandle<()>>,
}

impl Thread {
    pub fn new(chunk: Arc<Chunk>) -> Self {
        let mut call_stack = Vec::with_capacity(chunk.prototypes.len() + 1);
        let main_call = CallFrame::new(Arc::clone(&chunk), 0);

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
                Operation::MOVE => {
                    let source = instruction.b_field() as usize;
                    let destination = instruction.a_field() as usize;
                    let source_type = instruction.b_type();

                    match source_type {
                        TypeCode::BOOLEAN => {
                            let value = current_frame.registers.booleans.get(source).copy_value();

                            current_frame
                                .registers
                                .booleans
                                .get_mut(destination)
                                .set(value);
                        }
                        TypeCode::BYTE => {
                            let value = current_frame.registers.bytes.get(source).copy_value();

                            current_frame
                                .registers
                                .bytes
                                .get_mut(destination)
                                .set(value);
                        }
                        TypeCode::CHARACTER => {
                            let value = current_frame.registers.characters.get(source).copy_value();

                            current_frame
                                .registers
                                .characters
                                .get_mut(destination)
                                .set(value);
                        }
                        TypeCode::FLOAT => {
                            let value = current_frame.registers.floats.get(source).copy_value();

                            current_frame
                                .registers
                                .floats
                                .get_mut(destination)
                                .set(value);
                        }
                        TypeCode::INTEGER => {
                            let value = current_frame.registers.integers.get(source).copy_value();

                            current_frame
                                .registers
                                .integers
                                .get_mut(destination)
                                .set(value);
                        }
                        TypeCode::STRING => {
                            let value = current_frame.registers.strings.get(source).clone_value();

                            current_frame
                                .registers
                                .strings
                                .get_mut(destination)
                                .set(value);
                        }
                        TypeCode::LIST => {
                            let value = current_frame.registers.lists.get(source).clone_value();

                            current_frame
                                .registers
                                .lists
                                .get_mut(destination)
                                .set(value);
                        }
                        _ => todo!(),
                    }
                }
                Operation::CLOSE => {
                    let from = instruction.b_field() as usize;
                    let to = instruction.c_field() as usize;
                    let r#type = instruction.b_type();

                    match r#type {
                        TypeCode::BOOLEAN => {
                            let registers =
                                current_frame.registers.booleans.get_many_mut(from..=to);

                            for register in registers {
                                register.close();
                            }
                        }
                        TypeCode::BYTE => {
                            let registers = current_frame.registers.bytes.get_many_mut(from..=to);

                            for register in registers {
                                register.close();
                            }
                        }
                        TypeCode::CHARACTER => {
                            let registers =
                                current_frame.registers.characters.get_many_mut(from..=to);

                            for register in registers {
                                register.close();
                            }
                        }
                        TypeCode::FLOAT => {
                            let registers = current_frame.registers.floats.get_many_mut(from..=to);

                            for register in registers {
                                register.close();
                            }
                        }
                        TypeCode::INTEGER => {
                            let registers =
                                current_frame.registers.integers.get_many_mut(from..=to);

                            for register in registers {
                                register.close();
                            }
                        }
                        TypeCode::STRING => {
                            let registers = current_frame.registers.strings.get_many_mut(from..=to);

                            for register in registers {
                                register.close();
                            }
                        }
                        TypeCode::LIST => {
                            let registers = current_frame.registers.lists.get_many_mut(from..=to);

                            for register in registers {
                                register.close();
                            }
                        }
                        _ => unreachable!("Invalid CLOSE operation"),
                    }
                }
                Operation::LOAD_ENCODED => {
                    let destination = instruction.a_field() as usize;
                    let value_type = instruction.b_type();
                    let jump_next = instruction.c_field() != 0;

                    match value_type {
                        TypeCode::BOOLEAN => {
                            let boolean = instruction.b_field() != 0;

                            current_frame
                                .registers
                                .booleans
                                .set_to_new_register(destination, boolean);
                        }
                        TypeCode::BYTE => {
                            let byte = instruction.b_field() as u8;

                            current_frame
                                .registers
                                .bytes
                                .set_to_new_register(destination, byte);
                        }
                        _ => unreachable!(),
                    }

                    if jump_next {
                        current_frame.ip += 1;
                    }
                }
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
                Operation::LOAD_LIST => {
                    let destination = instruction.a_field() as usize;
                    let start_register = instruction.b_field() as usize;
                    let item_type = instruction.b_type();
                    let end_register = instruction.c_field() as usize;
                    let jump_next = instruction.d_field();

                    let mut item_pointers = Vec::with_capacity(end_register - start_register + 1);

                    match item_type {
                        TypeCode::BOOLEAN => {
                            for register_index in start_register..=end_register {
                                let register_is_closed =
                                    current_frame.registers.booleans.is_closed(register_index);

                                if register_is_closed {
                                    continue;
                                }

                                item_pointers.push(Pointer::Register(register_index));
                            }
                        }
                        TypeCode::BYTE => {
                            for register_index in start_register..=end_register {
                                let register_is_closed =
                                    current_frame.registers.bytes.is_closed(register_index);

                                if register_is_closed {
                                    continue;
                                }

                                item_pointers.push(Pointer::Register(register_index));
                            }
                        }
                        TypeCode::CHARACTER => {
                            for register_index in start_register..=end_register {
                                let register_is_closed =
                                    current_frame.registers.characters.is_closed(register_index);

                                if register_is_closed {
                                    continue;
                                }

                                item_pointers.push(Pointer::Register(register_index));
                            }
                        }
                        TypeCode::FLOAT => {
                            for register_index in start_register..=end_register {
                                let register_is_closed =
                                    current_frame.registers.floats.is_closed(register_index);

                                if register_is_closed {
                                    continue;
                                }

                                item_pointers.push(Pointer::Register(register_index));
                            }
                        }
                        TypeCode::INTEGER => {
                            for register_index in start_register..=end_register {
                                let register_is_closed =
                                    current_frame.registers.integers.is_closed(register_index);

                                if register_is_closed {
                                    continue;
                                }

                                item_pointers.push(Pointer::Register(register_index));
                            }
                        }
                        TypeCode::STRING => {
                            for register_index in start_register..=end_register {
                                let register_is_closed =
                                    current_frame.registers.strings.is_closed(register_index);

                                if register_is_closed {
                                    continue;
                                }

                                item_pointers.push(Pointer::Register(register_index));
                            }
                        }
                        TypeCode::LIST => {
                            for register_index in start_register..=end_register {
                                let register_is_closed =
                                    current_frame.registers.lists.is_closed(register_index);

                                if register_is_closed {
                                    continue;
                                }

                                item_pointers.push(Pointer::Register(register_index));
                            }
                        }
                        _ => unreachable!("Invalid LOAD_LIST operation"),
                    }

                    let list = AbstractList {
                        item_type,
                        item_pointers,
                    };

                    current_frame.registers.lists.get_mut(destination).set(list);

                    if jump_next {
                        current_frame.ip += 1;
                    }
                }
                Operation::LOAD_FUNCTION => {
                    let destination = instruction.a_field() as usize;
                    let prototype_index = instruction.b_field() as usize;
                    let jump_next = instruction.c_field() != 0;
                    let prototype = if cfg!(debug_assertions) {
                        current_frame.chunk.prototypes.get(prototype_index).unwrap()
                    } else {
                        unsafe {
                            current_frame
                                .chunk
                                .prototypes
                                .get_unchecked(prototype_index)
                        }
                    };
                    let function = prototype.as_function();

                    current_frame
                        .registers
                        .functions
                        .set_to_new_register(destination, function);

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
                            current_frame.registers.integers.get(left).copy_value()
                        };
                        let right_value = if right_is_constant {
                            current_frame.get_integer_constant(right)
                        } else {
                            current_frame.registers.integers.get(right).copy_value()
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

                        let left_value = current_frame.registers.bytes.get(left_index).copy_value();
                        let right_value =
                            current_frame.registers.bytes.get(right_index).copy_value();
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
                            current_frame
                                .registers
                                .characters
                                .get(left_index)
                                .copy_value()
                        };
                        let right_value = if right_is_constant {
                            current_frame.get_character_constant(right_index)
                        } else {
                            current_frame
                                .registers
                                .characters
                                .get(right_index)
                                .copy_value()
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
                            current_frame
                                .registers
                                .characters
                                .get(left_index)
                                .copy_value()
                        };
                        let right_value = if right_is_constant {
                            current_frame.get_string_constant(right_index)
                        } else {
                            current_frame.registers.strings.get(right_index).as_value()
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

                        let left_value =
                            current_frame.registers.floats.get(left_index).copy_value();
                        let right_value =
                            current_frame.registers.floats.get(right_index).copy_value();
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
                            current_frame.registers.integers.get(left).copy_value()
                        };
                        let right_value = if right_is_constant {
                            current_frame.get_integer_constant(right)
                        } else {
                            current_frame.registers.integers.get(right).copy_value()
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
                            current_frame.registers.strings.get(left).as_value()
                        };
                        let right_value = if right_is_constant {
                            current_frame.get_string_constant(right)
                        } else {
                            current_frame.registers.strings.get(right).as_value()
                        };
                        let concatenated = DustString::from(format!("{left_value}{right_value}"));

                        current_frame
                            .registers
                            .strings
                            .set_to_new_register(destination_index, concatenated);
                    }
                    _ => unreachable!("Invalid ADD operation"),
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
                                let return_value = current_frame
                                    .registers
                                    .booleans
                                    .get(return_register)
                                    .copy_value();

                                return Some(Value::boolean(return_value));
                            }
                            TypeCode::BYTE => {
                                let return_value = current_frame
                                    .registers
                                    .bytes
                                    .get(return_register)
                                    .copy_value();

                                return Some(Value::byte(return_value));
                            }
                            TypeCode::CHARACTER => {
                                let return_value = current_frame
                                    .registers
                                    .characters
                                    .get(return_register)
                                    .copy_value();

                                return Some(Value::character(return_value));
                            }
                            TypeCode::FLOAT => {
                                let return_value = current_frame
                                    .registers
                                    .floats
                                    .get(return_register)
                                    .copy_value();

                                return Some(Value::float(return_value));
                            }
                            TypeCode::INTEGER => {
                                let return_value = current_frame
                                    .registers
                                    .integers
                                    .get(return_register)
                                    .copy_value();

                                return Some(Value::integer(return_value));
                            }
                            TypeCode::STRING => {
                                let return_value = current_frame
                                    .registers
                                    .strings
                                    .get(return_register)
                                    .clone_value();

                                return Some(Value::string(return_value));
                            }
                            TypeCode::LIST => {
                                let concrete_list = current_frame
                                    .registers
                                    .lists
                                    .get(return_register)
                                    .as_value()
                                    .clone()
                                    .to_concrete(&self);

                                return Some(Value::Concrete(concrete_list));
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
