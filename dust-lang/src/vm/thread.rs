use std::{sync::Arc, thread::JoinHandle};

use tracing::{info, trace};

use crate::{
    instruction::TypeCode,
    vm::{CallFrame, Pointer},
    AbstractList, Chunk, DustString, NativeFunction, Operation, Span, Type, Value,
};

use super::RegisterTable;

pub struct Thread {
    chunk: Arc<Chunk>,
    call_stack: Vec<CallFrame>,
    register_stack: Vec<RegisterTable>,
    _spawned_threads: Vec<JoinHandle<()>>,
}

impl Thread {
    pub fn new(chunk: Arc<Chunk>) -> Self {
        let mut call_stack = Vec::with_capacity(chunk.prototypes.len() + 1);
        let main_call = CallFrame::new(Arc::clone(&chunk), 0);

        call_stack.push(main_call);

        let mut register_stack = Vec::with_capacity(chunk.prototypes.len() + 1);
        let main_registers = RegisterTable::new(&chunk);

        register_stack.push(main_registers);

        Thread {
            chunk,
            call_stack,
            register_stack,
            _spawned_threads: Vec::new(),
        }
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

    pub fn current_registers(&self) -> &RegisterTable {
        if cfg!(debug_assertions) {
            self.register_stack.last().unwrap()
        } else {
            unsafe { self.register_stack.last().unwrap_unchecked() }
        }
    }

    pub fn current_registers_mut(&mut self) -> &mut RegisterTable {
        if cfg!(debug_assertions) {
            self.register_stack.last_mut().unwrap()
        } else {
            unsafe { self.register_stack.last_mut().unwrap_unchecked() }
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

        loop {
            let current_frame = if cfg!(debug_assertions) {
                self.call_stack.last_mut().unwrap()
            } else {
                unsafe { self.call_stack.last_mut().unwrap_unchecked() }
            };
            let registers = if cfg!(debug_assertions) {
                self.register_stack.last_mut().unwrap()
            } else {
                unsafe { self.register_stack.last_mut().unwrap_unchecked() }
            };
            let instructions = &current_frame.chunk.instructions;
            let ip = current_frame.ip;
            current_frame.ip += 1;

            assert!(ip < instructions.len(), "IP out of bounds");

            let instruction = instructions[ip];

            info!("IP = {ip} Run {}", instruction.operation());

            match instruction.operation() {
                Operation::MOVE => {
                    let source = instruction.b_field() as usize;
                    let destination = instruction.a_field() as usize;
                    let source_type = instruction.b_type();

                    match source_type {
                        TypeCode::BOOLEAN => {
                            let value = registers.booleans.get(source).copy_value();

                            registers.booleans.get_mut(destination).set(value);
                        }
                        TypeCode::BYTE => {
                            let value = registers.bytes.get(source).copy_value();

                            registers.bytes.get_mut(destination).set(value);
                        }
                        TypeCode::CHARACTER => {
                            let value = registers.characters.get(source).copy_value();

                            registers.characters.get_mut(destination).set(value);
                        }
                        TypeCode::FLOAT => {
                            let value = registers.floats.get(source).copy_value();

                            registers.floats.get_mut(destination).set(value);
                        }
                        TypeCode::INTEGER => {
                            let value = registers.integers.get(source).copy_value();

                            registers.integers.get_mut(destination).set(value);
                        }
                        TypeCode::STRING => {
                            let value = registers.strings.get(source).clone_value();

                            registers.strings.get_mut(destination).set(value);
                        }
                        TypeCode::LIST => {
                            let value = registers.lists.get(source).clone_value();

                            registers.lists.get_mut(destination).set(value);
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
                            let registers = registers.booleans.get_many_mut(from..=to);

                            for register in registers {
                                register.close();
                            }
                        }
                        TypeCode::BYTE => {
                            let registers = registers.bytes.get_many_mut(from..=to);

                            for register in registers {
                                register.close();
                            }
                        }
                        TypeCode::CHARACTER => {
                            let registers = registers.characters.get_many_mut(from..=to);

                            for register in registers {
                                register.close();
                            }
                        }
                        TypeCode::FLOAT => {
                            let registers = registers.floats.get_many_mut(from..=to);

                            for register in registers {
                                register.close();
                            }
                        }
                        TypeCode::INTEGER => {
                            let registers = registers.integers.get_many_mut(from..=to);

                            for register in registers {
                                register.close();
                            }
                        }
                        TypeCode::STRING => {
                            let registers = registers.strings.get_many_mut(from..=to);

                            for register in registers {
                                register.close();
                            }
                        }
                        TypeCode::LIST => {
                            let registers = registers.lists.get_many_mut(from..=to);

                            for register in registers {
                                register.close();
                            }
                        }
                        _ => unreachable!("Invalid CLOSE instruction"),
                    }
                }
                Operation::LOAD_ENCODED => {
                    let destination = instruction.a_field() as usize;
                    let value_type = instruction.b_type();
                    let jump_next = instruction.c_field() != 0;

                    match value_type {
                        TypeCode::BOOLEAN => {
                            let boolean = instruction.b_field() != 0;

                            registers.booleans.set_to_new_register(destination, boolean);
                        }
                        TypeCode::BYTE => {
                            let byte = instruction.b_field() as u8;

                            registers.bytes.set_to_new_register(destination, byte);
                        }
                        _ => unreachable!("Invalid LOAD_ENCODED instruction"),
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

                            registers.characters.get_mut(destination).set(character);
                        }
                        TypeCode::FLOAT => {
                            let float = current_frame.get_float_constant(constant_index);

                            registers.floats.get_mut(destination).set(float);
                        }
                        TypeCode::INTEGER => {
                            let integer = current_frame.get_integer_constant(constant_index);

                            registers.integers.get_mut(destination).set(integer);
                        }
                        TypeCode::STRING => {
                            let string = current_frame.get_string_constant(constant_index).clone();

                            registers.strings.get_mut(destination).set(string);
                        }
                        _ => unreachable!("Invalid LOAD_CONSTANT operation"),
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
                                    registers.booleans.is_closed(register_index);

                                if register_is_closed {
                                    continue;
                                }

                                item_pointers.push(Pointer::Register(register_index as u16));
                            }
                        }
                        TypeCode::BYTE => {
                            for register_index in start_register..=end_register {
                                let register_is_closed = registers.bytes.is_closed(register_index);

                                if register_is_closed {
                                    continue;
                                }

                                item_pointers.push(Pointer::Register(register_index as u16));
                            }
                        }
                        TypeCode::CHARACTER => {
                            for register_index in start_register..=end_register {
                                let register_is_closed =
                                    registers.characters.is_closed(register_index);

                                if register_is_closed {
                                    continue;
                                }

                                item_pointers.push(Pointer::Register(register_index as u16));
                            }
                        }
                        TypeCode::FLOAT => {
                            for register_index in start_register..=end_register {
                                let register_is_closed = registers.floats.is_closed(register_index);

                                if register_is_closed {
                                    continue;
                                }

                                item_pointers.push(Pointer::Register(register_index as u16));
                            }
                        }
                        TypeCode::INTEGER => {
                            for register_index in start_register..=end_register {
                                let register_is_closed =
                                    registers.integers.is_closed(register_index);

                                if register_is_closed {
                                    continue;
                                }

                                item_pointers.push(Pointer::Register(register_index as u16));
                            }
                        }
                        TypeCode::STRING => {
                            for register_index in start_register..=end_register {
                                let register_is_closed =
                                    registers.strings.is_closed(register_index);

                                if register_is_closed {
                                    continue;
                                }

                                item_pointers.push(Pointer::Register(register_index as u16));
                            }
                        }
                        TypeCode::LIST => {
                            for register_index in start_register..=end_register {
                                let register_is_closed = registers.lists.is_closed(register_index);

                                if register_is_closed {
                                    continue;
                                }

                                item_pointers.push(Pointer::Register(register_index as u16));
                            }
                        }
                        _ => unreachable!("Invalid LOAD_LIST instruction"),
                    }

                    let list = AbstractList {
                        item_type,
                        item_pointers,
                    };

                    registers.lists.get_mut(destination).set(list);

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

                    registers
                        .functions
                        .set_to_new_register(destination, function);

                    if jump_next {
                        current_frame.ip += 1;
                    }
                }
                Operation::LOAD_SELF => {
                    let destination = instruction.a_field() as usize;
                    let jump_next = instruction.c_field() != 0;
                    let self_function = current_frame.chunk.as_function();

                    registers
                        .functions
                        .set_to_new_register(destination, self_function);

                    if jump_next {
                        current_frame.ip += 1;
                    }
                }
                Operation::ADD => match (instruction.b_type(), instruction.c_type()) {
                    (TypeCode::BYTE, TypeCode::BYTE) => {
                        let left_index = instruction.b_field() as usize;
                        let right_index = instruction.c_field() as usize;
                        let destination_index = instruction.a_field() as usize;

                        let left_value = registers.bytes.get(left_index).copy_value();
                        let right_value = registers.bytes.get(right_index).copy_value();
                        let sum = left_value + right_value;

                        registers.bytes.set_to_new_register(destination_index, sum);
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
                            registers.characters.get(left_index).copy_value()
                        };
                        let right_value = if right_is_constant {
                            current_frame.get_character_constant(right_index)
                        } else {
                            registers.characters.get(right_index).copy_value()
                        };
                        let concatenated = {
                            let mut concatenated = DustString::from(String::with_capacity(2));

                            concatenated.push(left_value);
                            concatenated.push(right_value);

                            concatenated
                        };

                        registers
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
                            registers.characters.get(left_index).copy_value()
                        };
                        let right_value = if right_is_constant {
                            current_frame.get_string_constant(right_index)
                        } else {
                            registers.strings.get(right_index).as_value()
                        };
                        let concatenated = DustString::from(format!("{left_value}{right_value}"));

                        registers
                            .strings
                            .set_to_new_register(destination_index, concatenated);
                    }
                    (TypeCode::FLOAT, TypeCode::FLOAT) => {
                        let left_index = instruction.b_field() as usize;
                        let left_is_constant = instruction.b_is_constant();
                        let right_index = instruction.c_field() as usize;
                        let right_is_constant = instruction.c_is_constant();
                        let destination_index = instruction.a_field() as usize;

                        let left_value = if left_is_constant {
                            current_frame.get_float_constant(left_index)
                        } else {
                            registers.floats.get(left_index).copy_value()
                        };
                        let right_value = if right_is_constant {
                            current_frame.get_float_constant(right_index)
                        } else {
                            registers.floats.get(right_index).copy_value()
                        };
                        let sum = left_value + right_value;

                        registers.floats.set_to_new_register(destination_index, sum);
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
                            registers.integers.get(left).copy_value()
                        };
                        let right_value = if right_is_constant {
                            current_frame.get_integer_constant(right)
                        } else {
                            registers.integers.get(right).copy_value()
                        };
                        let sum = left_value + right_value;

                        registers
                            .integers
                            .set_to_new_register(destination_index, sum);
                    }
                    (TypeCode::STRING, TypeCode::CHARACTER) => {
                        let left_index = instruction.b_field() as usize;
                        let left_is_constant = instruction.b_is_constant();
                        let right_index = instruction.c_field() as usize;
                        let right_is_constant = instruction.c_is_constant();
                        let destination_index = instruction.a_field() as usize;

                        let left_value = if left_is_constant {
                            current_frame.get_string_constant(left_index)
                        } else {
                            registers.strings.get(left_index).as_value()
                        };
                        let right_value = if right_is_constant {
                            current_frame.get_character_constant(right_index)
                        } else {
                            registers.characters.get(right_index).copy_value()
                        };
                        let concatenated = DustString::from(format!("{left_value}{right_value}"));

                        registers
                            .strings
                            .set_to_new_register(destination_index, concatenated);
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
                            registers.strings.get(left).as_value()
                        };
                        let right_value = if right_is_constant {
                            current_frame.get_string_constant(right)
                        } else {
                            registers.strings.get(right).as_value()
                        };
                        let concatenated = DustString::from(format!("{left_value}{right_value}"));

                        registers
                            .strings
                            .set_to_new_register(destination_index, concatenated);
                    }
                    _ => unreachable!("Invalid ADD instruction"),
                },
                Operation::SUBTRACT => match (instruction.b_type(), instruction.c_type()) {
                    (TypeCode::BYTE, TypeCode::BYTE) => {
                        let left_index = instruction.b_field() as usize;
                        let right_index = instruction.c_field() as usize;
                        let destination_index = instruction.a_field() as usize;

                        let left_value = registers.bytes.get(left_index).copy_value();
                        let right_value = registers.bytes.get(right_index).copy_value();
                        let difference = left_value - right_value;

                        registers
                            .bytes
                            .set_to_new_register(destination_index, difference);
                    }
                    (TypeCode::FLOAT, TypeCode::FLOAT) => {
                        let left_index = instruction.b_field() as usize;
                        let left_is_constant = instruction.b_is_constant();
                        let right_index = instruction.c_field() as usize;
                        let right_is_constant = instruction.c_is_constant();
                        let destination_index = instruction.a_field() as usize;

                        let left_value = if left_is_constant {
                            current_frame.get_float_constant(left_index)
                        } else {
                            registers.floats.get(left_index).copy_value()
                        };
                        let right_value = if right_is_constant {
                            current_frame.get_float_constant(right_index)
                        } else {
                            registers.floats.get(right_index).copy_value()
                        };
                        let difference = left_value - right_value;

                        registers
                            .floats
                            .set_to_new_register(destination_index, difference);
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
                            registers.integers.get(left).copy_value()
                        };
                        let right_value = if right_is_constant {
                            current_frame.get_integer_constant(right)
                        } else {
                            registers.integers.get(right).copy_value()
                        };
                        let difference = left_value - right_value;

                        registers
                            .integers
                            .set_to_new_register(destination_index, difference);
                    }
                    _ => unreachable!("Invalid SUBTRACT instruction"),
                },
                Operation::MULTIPLY => match (instruction.b_type(), instruction.c_type()) {
                    (TypeCode::BYTE, TypeCode::BYTE) => {
                        let left_index = instruction.b_field() as usize;
                        let right_index = instruction.c_field() as usize;
                        let destination_index = instruction.a_field() as usize;

                        let left_value = registers.bytes.get(left_index).copy_value();
                        let right_value = registers.bytes.get(right_index).copy_value();
                        let product = left_value * right_value;

                        registers
                            .bytes
                            .set_to_new_register(destination_index, product);
                    }
                    (TypeCode::FLOAT, TypeCode::FLOAT) => {
                        let left_index = instruction.b_field() as usize;
                        let left_is_constant = instruction.b_is_constant();
                        let right_index = instruction.c_field() as usize;
                        let right_is_constant = instruction.c_is_constant();
                        let destination_index = instruction.a_field() as usize;

                        let left_value = if left_is_constant {
                            current_frame.get_float_constant(left_index)
                        } else {
                            registers.floats.get(left_index).copy_value()
                        };
                        let right_value = if right_is_constant {
                            current_frame.get_float_constant(right_index)
                        } else {
                            registers.floats.get(right_index).copy_value()
                        };
                        let product = left_value * right_value;

                        registers
                            .floats
                            .set_to_new_register(destination_index, product);
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
                            registers.integers.get(left).copy_value()
                        };
                        let right_value = if right_is_constant {
                            current_frame.get_integer_constant(right)
                        } else {
                            registers.integers.get(right).copy_value()
                        };
                        let product = left_value * right_value;

                        registers
                            .integers
                            .set_to_new_register(destination_index, product);
                    }
                    _ => unreachable!("Invalid MULTIPLY instruction"),
                },
                Operation::DIVIDE => match (instruction.b_type(), instruction.c_type()) {
                    (TypeCode::BYTE, TypeCode::BYTE) => {
                        let left_index = instruction.b_field() as usize;
                        let right_index = instruction.c_field() as usize;
                        let destination_index = instruction.a_field() as usize;

                        let left_value = registers.bytes.get(left_index).copy_value();
                        let right_value = registers.bytes.get(right_index).copy_value();
                        let quotient = left_value / right_value;

                        registers
                            .bytes
                            .set_to_new_register(destination_index, quotient);
                    }
                    (TypeCode::FLOAT, TypeCode::FLOAT) => {
                        let left_index = instruction.b_field() as usize;
                        let left_is_constant = instruction.b_is_constant();
                        let right_index = instruction.c_field() as usize;
                        let right_is_constant = instruction.c_is_constant();
                        let destination_index = instruction.a_field() as usize;

                        let left_value = if left_is_constant {
                            current_frame.get_float_constant(left_index)
                        } else {
                            registers.floats.get(left_index).copy_value()
                        };
                        let right_value = if right_is_constant {
                            current_frame.get_float_constant(right_index)
                        } else {
                            registers.floats.get(right_index).copy_value()
                        };
                        let quotient = left_value / right_value;

                        registers
                            .floats
                            .set_to_new_register(destination_index, quotient);
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
                            registers.integers.get(left).copy_value()
                        };
                        let right_value = if right_is_constant {
                            current_frame.get_integer_constant(right)
                        } else {
                            registers.integers.get(right).copy_value()
                        };
                        let quotient = left_value / right_value;

                        registers
                            .integers
                            .set_to_new_register(destination_index, quotient);
                    }
                    _ => unreachable!("Invalid DIVIDE instruction"),
                },
                Operation::MODULO => match (instruction.b_type(), instruction.c_type()) {
                    (TypeCode::BYTE, TypeCode::BYTE) => {
                        let left_index = instruction.b_field() as usize;
                        let right_index = instruction.c_field() as usize;
                        let destination_index = instruction.a_field() as usize;

                        let left_value = registers.bytes.get(left_index).copy_value();
                        let right_value = registers.bytes.get(right_index).copy_value();
                        let remainder = left_value % right_value;

                        registers
                            .bytes
                            .set_to_new_register(destination_index, remainder);
                    }
                    (TypeCode::FLOAT, TypeCode::FLOAT) => {
                        let left_index = instruction.b_field() as usize;
                        let left_is_constant = instruction.b_is_constant();
                        let right_index = instruction.c_field() as usize;
                        let right_is_constant = instruction.c_is_constant();
                        let destination_index = instruction.a_field() as usize;

                        let left_value = if left_is_constant {
                            current_frame.get_float_constant(left_index)
                        } else {
                            registers.floats.get(left_index).copy_value()
                        };
                        let right_value = if right_is_constant {
                            current_frame.get_float_constant(right_index)
                        } else {
                            registers.floats.get(right_index).copy_value()
                        };
                        let remainder = left_value % right_value;

                        registers
                            .floats
                            .set_to_new_register(destination_index, remainder);
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
                            registers.integers.get(left).copy_value()
                        };
                        let right_value = if right_is_constant {
                            current_frame.get_integer_constant(right)
                        } else {
                            registers.integers.get(right).copy_value()
                        };
                        let remainder = left_value % right_value;

                        registers
                            .integers
                            .set_to_new_register(destination_index, remainder);
                    }
                    _ => unreachable!("Invalid MODULO instruction"),
                },
                Operation::EQUAL => match (instruction.b_type(), instruction.c_type()) {
                    (TypeCode::BOOLEAN, TypeCode::BOOLEAN) => {
                        let left_index = instruction.b_field() as usize;
                        let right_index = instruction.c_field() as usize;
                        let comparator = instruction.d_field();

                        let left_value = registers.booleans.get(left_index).copy_value();
                        let right_value = registers.booleans.get(right_index).copy_value();

                        // See <https://github.com/rust-lang/rust/issues/66780> for more info.
                        let is_equal = matches!((left_value as i8) - (right_value as i8), 0);

                        if is_equal == comparator {
                            current_frame.ip += 1;
                        }
                    }
                    (TypeCode::BYTE, TypeCode::BYTE) => {
                        let left = instruction.b_field() as usize;
                        let right = instruction.c_field() as usize;
                        let comparator = instruction.d_field();

                        let left_value = registers.bytes.get(left).copy_value();
                        let right_value = registers.bytes.get(right).copy_value();
                        let is_equal = left_value == right_value;

                        if is_equal == comparator {
                            current_frame.ip += 1;
                        }
                    }
                    (TypeCode::CHARACTER, TypeCode::CHARACTER) => {
                        let left = instruction.b_field() as usize;
                        let left_is_constant = instruction.b_is_constant();
                        let right = instruction.c_field() as usize;
                        let right_is_constant = instruction.c_is_constant();
                        let comparator = instruction.d_field();

                        let left_value = if left_is_constant {
                            current_frame.get_character_constant(left)
                        } else {
                            registers.characters.get(left).copy_value()
                        };
                        let right_value = if right_is_constant {
                            current_frame.get_character_constant(right)
                        } else {
                            registers.characters.get(right).copy_value()
                        };
                        let is_equal = left_value == right_value;

                        if is_equal == comparator {
                            current_frame.ip += 1;
                        }
                    }
                    (TypeCode::FLOAT, TypeCode::FLOAT) => {
                        let left = instruction.b_field() as usize;
                        let left_is_constant = instruction.b_is_constant();
                        let right = instruction.c_field() as usize;
                        let right_is_constant = instruction.c_is_constant();
                        let comparator = instruction.d_field();

                        let left_value = if left_is_constant {
                            current_frame.get_float_constant(left)
                        } else {
                            registers.floats.get(left).copy_value()
                        };
                        let right_value = if right_is_constant {
                            current_frame.get_float_constant(right)
                        } else {
                            registers.floats.get(right).copy_value()
                        };
                        let is_equal = left_value == right_value;

                        if is_equal == comparator {
                            current_frame.ip += 1;
                        }
                    }
                    (TypeCode::INTEGER, TypeCode::INTEGER) => {
                        let left = instruction.b_field() as usize;
                        let left_is_constant = instruction.b_is_constant();
                        let right = instruction.c_field() as usize;
                        let right_is_constant = instruction.c_is_constant();
                        let comparator = instruction.d_field();

                        let left_value = if left_is_constant {
                            current_frame.get_integer_constant(left)
                        } else {
                            registers.integers.get(left).copy_value()
                        };
                        let right_value = if right_is_constant {
                            current_frame.get_integer_constant(right)
                        } else {
                            registers.integers.get(right).copy_value()
                        };
                        let is_equal = left_value == right_value;

                        if is_equal == comparator {
                            current_frame.ip += 1;
                        }
                    }
                    (TypeCode::STRING, TypeCode::STRING) => {
                        let left = instruction.b_field() as usize;
                        let left_is_constant = instruction.b_is_constant();
                        let right = instruction.c_field() as usize;
                        let right_is_constant = instruction.c_is_constant();
                        let comparator = instruction.d_field();

                        let left_value = if left_is_constant {
                            current_frame.get_string_constant(left)
                        } else {
                            registers.strings.get(left).as_value()
                        };
                        let right_value = if right_is_constant {
                            current_frame.get_string_constant(right)
                        } else {
                            registers.strings.get(right).as_value()
                        };
                        let is_equal = left_value == right_value;

                        if is_equal == comparator {
                            current_frame.ip += 1;
                        }
                    }
                    (TypeCode::LIST, TypeCode::LIST) => {
                        let left = instruction.b_field() as usize;
                        let right = instruction.c_field() as usize;
                        let comparator = instruction.d_field();

                        let left_value = registers.lists.get(left).as_value();
                        let right_value = registers.lists.get(right).as_value();
                        let is_equal = left_value == right_value;

                        if is_equal == comparator {
                            current_frame.ip += 1;
                        }
                    }
                    (TypeCode::FUNCTION, TypeCode::FUNCTION) => {
                        let left = instruction.b_field() as usize;
                        let right = instruction.c_field() as usize;
                        let comparator = instruction.d_field();

                        let left_value = registers.functions.get(left).as_value();
                        let right_value = registers.functions.get(right).as_value();
                        let is_equal = left_value == right_value;

                        if is_equal == comparator {
                            current_frame.ip += 1;
                        }
                    }
                    _ => unreachable!("Invalid EQUAL instruction"),
                },
                Operation::LESS => match (instruction.b_type(), instruction.c_type()) {
                    (TypeCode::BOOLEAN, TypeCode::BOOLEAN) => {
                        let left_index = instruction.b_field() as usize;
                        let right_index = instruction.c_field() as usize;
                        let comparator = instruction.d_field();

                        let left_value = registers.booleans.get(left_index).copy_value();
                        let right_value = registers.booleans.get(right_index).copy_value();

                        // See <https://github.com/rust-lang/rust/issues/66780> for more info.
                        let is_less_than = matches!((left_value as i8) - (right_value as i8), -1);

                        if is_less_than == comparator {
                            current_frame.ip += 1;
                        }
                    }
                    (TypeCode::BYTE, TypeCode::BYTE) => {
                        let left = instruction.b_field() as usize;
                        let right = instruction.c_field() as usize;
                        let comparator = instruction.d_field();

                        let left_value = registers.bytes.get(left).copy_value();
                        let right_value = registers.bytes.get(right).copy_value();
                        let is_less_than = left_value < right_value;

                        if is_less_than == comparator {
                            current_frame.ip += 1;
                        }
                    }
                    (TypeCode::CHARACTER, TypeCode::CHARACTER) => {
                        let left = instruction.b_field() as usize;
                        let left_is_constant = instruction.b_is_constant();
                        let right = instruction.c_field() as usize;
                        let right_is_constant = instruction.c_is_constant();
                        let comparator = instruction.d_field();

                        let left_value = if left_is_constant {
                            current_frame.get_character_constant(left)
                        } else {
                            registers.characters.get(left).copy_value()
                        };
                        let right_value = if right_is_constant {
                            current_frame.get_character_constant(right)
                        } else {
                            registers.characters.get(right).copy_value()
                        };
                        let is_less_than = left_value < right_value;

                        if is_less_than == comparator {
                            current_frame.ip += 1;
                        }
                    }
                    (TypeCode::FLOAT, TypeCode::FLOAT) => {
                        let left = instruction.b_field() as usize;
                        let left_is_constant = instruction.b_is_constant();
                        let right = instruction.c_field() as usize;
                        let right_is_constant = instruction.c_is_constant();
                        let comparator = instruction.d_field();

                        let left_value = if left_is_constant {
                            current_frame.get_float_constant(left)
                        } else {
                            registers.floats.get(left).copy_value()
                        };
                        let right_value = if right_is_constant {
                            current_frame.get_float_constant(right)
                        } else {
                            registers.floats.get(right).copy_value()
                        };
                        let is_less_than = left_value < right_value;

                        if is_less_than == comparator {
                            current_frame.ip += 1;
                        }
                    }
                    (TypeCode::INTEGER, TypeCode::INTEGER) => {
                        let left = instruction.b_field() as usize;
                        let left_is_constant = instruction.b_is_constant();
                        let right = instruction.c_field() as usize;
                        let right_is_constant = instruction.c_is_constant();
                        let comparator = instruction.d_field();

                        let left_value = if left_is_constant {
                            current_frame.get_integer_constant(left)
                        } else {
                            registers.integers.get(left).copy_value()
                        };
                        let right_value = if right_is_constant {
                            current_frame.get_integer_constant(right)
                        } else {
                            registers.integers.get(right).copy_value()
                        };
                        let is_less_than = left_value < right_value;

                        if is_less_than == comparator {
                            current_frame.ip += 1;
                        }
                    }
                    (TypeCode::STRING, TypeCode::STRING) => {
                        let left = instruction.b_field() as usize;
                        let left_is_constant = instruction.b_is_constant();
                        let right = instruction.c_field() as usize;
                        let right_is_constant = instruction.c_is_constant();
                        let comparator = instruction.d_field();

                        let left_value = if left_is_constant {
                            current_frame.get_string_constant(left)
                        } else {
                            registers.strings.get(left).as_value()
                        };
                        let right_value = if right_is_constant {
                            current_frame.get_string_constant(right)
                        } else {
                            registers.strings.get(right).as_value()
                        };
                        let is_less_than = left_value < right_value;

                        if is_less_than == comparator {
                            current_frame.ip += 1;
                        }
                    }
                    (TypeCode::LIST, TypeCode::LIST) => {
                        let left = instruction.b_field() as usize;
                        let right = instruction.c_field() as usize;
                        let comparator = instruction.d_field();

                        let left_value = registers.lists.get(left).as_value();
                        let right_value = registers.lists.get(right).as_value();
                        let is_less_than = left_value < right_value;

                        if is_less_than == comparator {
                            current_frame.ip += 1;
                        }
                    }
                    (TypeCode::FUNCTION, TypeCode::FUNCTION) => {
                        let left = instruction.b_field() as usize;
                        let right = instruction.c_field() as usize;
                        let comparator = instruction.d_field();

                        let left_value = registers.functions.get(left).as_value();
                        let right_value = registers.functions.get(right).as_value();
                        let is_less_than = left_value < right_value;

                        if is_less_than == comparator {
                            current_frame.ip += 1;
                        }
                    }
                    _ => unreachable!("Invalid LESS instruction"),
                },
                Operation::LESS_EQUAL => match (instruction.b_type(), instruction.c_type()) {
                    (TypeCode::BOOLEAN, TypeCode::BOOLEAN) => {
                        let left_index = instruction.b_field() as usize;
                        let right_index = instruction.c_field() as usize;
                        let comparator = instruction.d_field();

                        let left_value = registers.booleans.get(left_index).copy_value();
                        let right_value = registers.booleans.get(right_index).copy_value();

                        // See <https://github.com/rust-lang/rust/issues/66780> for more info.
                        let is_less_than_or_equal =
                            matches!(left_value as i8 - right_value as i8, -1 | 0);

                        if is_less_than_or_equal == comparator {
                            current_frame.ip += 1;
                        }
                    }
                    (TypeCode::BYTE, TypeCode::BYTE) => {
                        let left = instruction.b_field() as usize;
                        let right = instruction.c_field() as usize;
                        let comparator = instruction.d_field();

                        let left_value = registers.bytes.get(left).copy_value();
                        let right_value = registers.bytes.get(right).copy_value();
                        let is_less_than_or_equal = left_value <= right_value;

                        if is_less_than_or_equal == comparator {
                            current_frame.ip += 1;
                        }
                    }
                    (TypeCode::CHARACTER, TypeCode::CHARACTER) => {
                        let left = instruction.b_field() as usize;
                        let left_is_constant = instruction.b_is_constant();
                        let right = instruction.c_field() as usize;
                        let right_is_constant = instruction.c_is_constant();
                        let comparator = instruction.d_field();

                        let left_value = if left_is_constant {
                            current_frame.get_character_constant(left)
                        } else {
                            registers.characters.get(left).copy_value()
                        };
                        let right_value = if right_is_constant {
                            current_frame.get_character_constant(right)
                        } else {
                            registers.characters.get(right).copy_value()
                        };
                        let is_less_than_or_equal = left_value <= right_value;

                        if is_less_than_or_equal == comparator {
                            current_frame.ip += 1;
                        }
                    }
                    (TypeCode::FLOAT, TypeCode::FLOAT) => {
                        let left = instruction.b_field() as usize;
                        let left_is_constant = instruction.b_is_constant();
                        let right = instruction.c_field() as usize;
                        let right_is_constant = instruction.c_is_constant();
                        let comparator = instruction.d_field();

                        let left_value = if left_is_constant {
                            current_frame.get_float_constant(left)
                        } else {
                            registers.floats.get(left).copy_value()
                        };
                        let right_value = if right_is_constant {
                            current_frame.get_float_constant(right)
                        } else {
                            registers.floats.get(right).copy_value()
                        };
                        let is_less_than_or_equal = left_value <= right_value;

                        if is_less_than_or_equal == comparator {
                            current_frame.ip += 1;
                        }
                    }
                    (TypeCode::INTEGER, TypeCode::INTEGER) => {
                        let left = instruction.b_field() as usize;
                        let left_is_constant = instruction.b_is_constant();
                        let right = instruction.c_field() as usize;
                        let right_is_constant = instruction.c_is_constant();
                        let comparator = instruction.d_field();

                        let left_value = if left_is_constant {
                            current_frame.get_integer_constant(left)
                        } else {
                            registers.integers.get(left).copy_value()
                        };
                        let right_value = if right_is_constant {
                            current_frame.get_integer_constant(right)
                        } else {
                            registers.integers.get(right).copy_value()
                        };
                        let is_less_than_or_equal = left_value <= right_value;

                        if is_less_than_or_equal == comparator {
                            current_frame.ip += 1;
                        }
                    }
                    (TypeCode::STRING, TypeCode::STRING) => {
                        let left = instruction.b_field() as usize;
                        let left_is_constant = instruction.b_is_constant();
                        let right = instruction.c_field() as usize;
                        let right_is_constant = instruction.c_is_constant();
                        let comparator = instruction.d_field();

                        let left_value = if left_is_constant {
                            current_frame.get_string_constant(left)
                        } else {
                            registers.strings.get(left).as_value()
                        };
                        let right_value = if right_is_constant {
                            current_frame.get_string_constant(right)
                        } else {
                            registers.strings.get(right).as_value()
                        };
                        let is_less_than_or_equal = left_value <= right_value;

                        if is_less_than_or_equal == comparator {
                            current_frame.ip += 1;
                        }
                    }
                    (TypeCode::LIST, TypeCode::LIST) => {
                        let left = instruction.b_field() as usize;
                        let right = instruction.c_field() as usize;
                        let comparator = instruction.d_field();

                        let left_value = registers.lists.get(left).as_value();
                        let right_value = registers.lists.get(right).as_value();
                        let is_less_than_or_equal = left_value <= right_value;

                        if is_less_than_or_equal == comparator {
                            current_frame.ip += 1;
                        }
                    }
                    (TypeCode::FUNCTION, TypeCode::FUNCTION) => {
                        let left = instruction.b_field() as usize;
                        let right = instruction.c_field() as usize;
                        let comparator = instruction.d_field();

                        let left_value = registers.functions.get(left).as_value();
                        let right_value = registers.functions.get(right).as_value();
                        let is_less_than_or_equal = left_value <= right_value;

                        if is_less_than_or_equal == comparator {
                            current_frame.ip += 1;
                        }
                    }
                    _ => unreachable!("Invalid LESS_EQUAL instruction"),
                },
                Operation::TEST => {
                    let operand_register_index = instruction.b_field() as usize;
                    let test_value = instruction.c_field() != 0;
                    let operand_boolean =
                        registers.booleans.get(operand_register_index).copy_value();

                    if operand_boolean == test_value {
                        current_frame.ip += 1;
                    }
                }
                Operation::CALL => {
                    let destination = instruction.a_field();
                    let function_register = instruction.b_field();
                    let argument_list_register = instruction.c_field();
                    let is_recursive = instruction.b_is_constant();

                    let function = if is_recursive {
                        current_frame.chunk.as_function()
                    } else {
                        registers
                            .functions
                            .get(function_register as usize)
                            .as_value()
                            .clone()
                    };
                    let function_prototype = if is_recursive {
                        &current_frame.chunk
                    } else {
                        current_frame
                            .chunk
                            .prototypes
                            .get(function.prototype_index as usize)
                            .unwrap()
                    };
                    let argument_list = current_frame
                        .chunk
                        .argument_lists
                        .get(argument_list_register as usize)
                        .unwrap();
                    let call_frame = CallFrame {
                        chunk: Arc::clone(function_prototype),
                        ip: 0,
                        return_register: destination,
                    };
                    let mut new_registers = RegisterTable::new(function_prototype);

                    for (r#type, register_index) in function
                        .r#type
                        .value_parameters
                        .iter()
                        .zip(argument_list.0.iter())
                    {
                        let register_index = *register_index as usize;

                        match r#type {
                            Type::Boolean => {
                                let boolean = *registers.booleans.get(register_index).as_value();

                                *new_registers
                                    .booleans
                                    .get_mut(register_index)
                                    .as_value_mut() = boolean;
                            }
                            Type::Integer => {
                                let integer = *registers.integers.get(register_index).as_value();

                                *new_registers
                                    .integers
                                    .get_mut(register_index)
                                    .as_value_mut() = integer;
                            }
                            Type::String => {
                                let string =
                                    registers.strings.get(register_index).as_value().clone();

                                *new_registers.strings.get_mut(register_index).as_value_mut() =
                                    string;
                            }
                            _ => unreachable!(),
                        }
                    }

                    self.call_stack.push(call_frame);
                    self.register_stack.push(new_registers);

                    trace!("Call Stack: {:?}", self.call_stack);
                }
                Operation::CALL_NATIVE => {
                    let function = NativeFunction::from(instruction.b_field());

                    function.call(instruction, &mut self);
                }
                Operation::CALL_NATIVE => {
                    let function = NativeFunction::from(instruction.b_field());

                    function.call(instruction, &mut self);
                }
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

                    if self.call_stack.len() == 1 {
                        if should_return_value {
                            match return_type {
                                TypeCode::BOOLEAN => {
                                    let return_value =
                                        registers.booleans.get(return_register).copy_value();

                                    return Some(Value::boolean(return_value));
                                }
                                TypeCode::BYTE => {
                                    let return_value =
                                        registers.bytes.get(return_register).copy_value();

                                    return Some(Value::byte(return_value));
                                }
                                TypeCode::CHARACTER => {
                                    let return_value =
                                        registers.characters.get(return_register).copy_value();

                                    return Some(Value::character(return_value));
                                }
                                TypeCode::FLOAT => {
                                    let return_value =
                                        registers.floats.get(return_register).copy_value();

                                    return Some(Value::float(return_value));
                                }
                                TypeCode::INTEGER => {
                                    let return_value =
                                        registers.integers.get(return_register).copy_value();

                                    return Some(Value::integer(return_value));
                                }
                                TypeCode::STRING => {
                                    let return_value =
                                        registers.strings.get(return_register).clone_value();

                                    return Some(Value::string(return_value));
                                }
                                TypeCode::LIST => {
                                    let concrete_list = registers
                                        .lists
                                        .get(return_register)
                                        .as_value()
                                        .clone()
                                        .to_concrete(&self);

                                    return Some(Value::Concrete(concrete_list));
                                }
                                TypeCode::FUNCTION => {
                                    let return_value =
                                        registers.functions.get(return_register).clone_value();

                                    return Some(Value::Function(return_value));
                                }
                                _ => unreachable!(),
                            }
                        } else {
                            return None;
                        }
                    } else if should_return_value {
                        match return_type {
                            TypeCode::BOOLEAN => {
                                let return_value =
                                    registers.booleans.get(return_register).copy_value();
                                let return_register =
                                    self.call_stack.last().unwrap().return_register as usize;

                                self.call_stack.pop();
                                self.register_stack.pop();

                                let registers = self.register_stack.last_mut().unwrap();

                                *registers.booleans.get_mut(return_register).as_value_mut() =
                                    return_value;
                            }
                            TypeCode::BYTE => {
                                let return_value =
                                    registers.bytes.get(return_register).copy_value();
                                let return_register =
                                    self.call_stack.last().unwrap().return_register as usize;

                                self.call_stack.pop();
                                self.register_stack.pop();

                                let registers = self.register_stack.last_mut().unwrap();

                                *registers.bytes.get_mut(return_register).as_value_mut() =
                                    return_value;
                            }
                            TypeCode::CHARACTER => {
                                let return_value =
                                    registers.characters.get(return_register).copy_value();
                                let return_register =
                                    self.call_stack.last().unwrap().return_register as usize;

                                self.call_stack.pop();
                                self.register_stack.pop();

                                let registers = self.register_stack.last_mut().unwrap();

                                *registers.characters.get_mut(return_register).as_value_mut() =
                                    return_value;
                            }
                            TypeCode::FLOAT => {
                                let return_value =
                                    registers.floats.get(return_register).copy_value();
                                let return_register =
                                    self.call_stack.last().unwrap().return_register as usize;

                                self.call_stack.pop();
                                self.register_stack.pop();

                                let registers = self.register_stack.last_mut().unwrap();

                                *registers.floats.get_mut(return_register).as_value_mut() =
                                    return_value;
                            }
                            TypeCode::INTEGER => {
                                let return_value =
                                    registers.integers.get(return_register).copy_value();
                                let return_register =
                                    self.call_stack.last().unwrap().return_register as usize;

                                self.call_stack.pop();
                                self.register_stack.pop();

                                let registers = self.register_stack.last_mut().unwrap();

                                *registers.integers.get_mut(return_register).as_value_mut() =
                                    return_value;
                            }
                            TypeCode::STRING => {
                                let return_value =
                                    registers.strings.get(return_register).clone_value();
                                let return_register =
                                    self.call_stack.last().unwrap().return_register as usize;

                                self.call_stack.pop();
                                self.register_stack.pop();

                                let registers = self.register_stack.last_mut().unwrap();

                                *registers.strings.get_mut(return_register).as_value_mut() =
                                    return_value;
                            }
                            TypeCode::LIST => {
                                let return_value =
                                    registers.lists.get(return_register).as_value().clone();
                                let return_register =
                                    self.call_stack.last().unwrap().return_register as usize;

                                self.call_stack.pop();
                                self.register_stack.pop();

                                let registers = self.register_stack.last_mut().unwrap();

                                *registers.lists.get_mut(return_register).as_value_mut() =
                                    return_value;
                            }
                            TypeCode::FUNCTION => {
                                let return_value =
                                    registers.functions.get(return_register).clone_value();
                                let return_register =
                                    self.call_stack.last().unwrap().return_register as usize;

                                self.call_stack.pop();
                                self.register_stack.pop();

                                let registers = self.register_stack.last_mut().unwrap();

                                *registers.functions.get_mut(return_register).as_value_mut() =
                                    return_value;
                            }
                            _ => unreachable!(),
                        }
                    }
                }
                unimplemented => {
                    todo!("{unimplemented} has not been implemented in the VM");
                }
            }
        }
    }
}
