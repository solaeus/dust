use std::{collections::HashMap, sync::Arc, thread::JoinHandle};

use tracing::{info, trace};

use crate::{Chunk, ConcreteValue, DustString, Span, Value, vm::CallFrame};

use super::call_frame::{Pointer, Register};

pub struct Thread {
    chunk: Arc<Chunk>,
    call_stack: Vec<CallFrame>,
    pub return_value: Option<Option<Value>>,
    pub integer_cache: HashMap<usize, *const i64>,
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
            return_value: None,
            integer_cache: HashMap::new(),
            _spawned_threads: Vec::new(),
        }
    }

    pub fn run(mut self) -> Option<Value> {
        info!(
            "Starting thread with {}",
            self.chunk
                .name
                .clone()
                .unwrap_or_else(|| DustString::from("anonymous"))
        );

        loop {
            let current_frame = self.current_frame_mut();
            let ip = {
                let ip = current_frame.ip;
                current_frame.ip += 1;

                ip
            };
            let current_action = if cfg!(debug_assertions) {
                current_frame.action_sequence.actions.get_mut(ip).unwrap()
            } else {
                unsafe { current_frame.action_sequence.actions.get_unchecked_mut(ip) }
            };

            trace!(
                "Instruction operation: {}",
                current_action.instruction.operation
            );

            (current_action.logic)(current_action.instruction, &mut self);

            if let Some(return_value_option) = self.return_value {
                return return_value_option;
            }
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

    pub fn get_boolean_register(&self, register_index: usize) -> &bool {
        let register = if cfg!(debug_assertions) {
            self.call_stack
                .last()
                .unwrap()
                .registers
                .booleans
                .get(register_index)
                .unwrap()
        } else {
            unsafe {
                self.call_stack
                    .last()
                    .unwrap_unchecked()
                    .registers
                    .booleans
                    .get_unchecked(register_index)
            }
        };

        match register {
            Register::Value(value) => value,
            Register::Pointer(pointer) => self.get_pointer_to_boolean(pointer),
            Register::Empty => panic!("Attempted to get value from empty register"),
        }
    }

    pub fn get_pointer_to_boolean(&self, pointer: &Pointer) -> &bool {
        match pointer {
            Pointer::Register(register_index) => self.get_boolean_register(*register_index),
            Pointer::Constant(constant_index) => {
                self.get_constant(*constant_index).as_boolean().unwrap()
            }
            Pointer::Stack(call_index, register_index) => {
                let call_frame = if cfg!(debug_assertions) {
                    self.call_stack.get(*call_index).unwrap()
                } else {
                    unsafe { self.call_stack.get_unchecked(*call_index) }
                };
                let register = if cfg!(debug_assertions) {
                    call_frame.registers.booleans.get(*register_index).unwrap()
                } else {
                    unsafe { call_frame.registers.booleans.get_unchecked(*register_index) }
                };

                match register {
                    Register::Value(value) => value,
                    Register::Pointer(pointer) => self.get_pointer_to_boolean(pointer),
                    Register::Empty => panic!("Attempted to get value from empty register"),
                }
            }
        }
    }

    pub fn set_boolean_register(&mut self, register_index: usize, new_register: Register<bool>) {
        let old_register = if cfg!(debug_assertions) {
            self.call_stack
                .last_mut()
                .unwrap()
                .registers
                .booleans
                .get_mut(register_index)
                .unwrap()
        } else {
            unsafe {
                self.call_stack
                    .last_mut()
                    .unwrap_unchecked()
                    .registers
                    .booleans
                    .get_unchecked_mut(register_index)
            }
        };

        *old_register = new_register;
    }

    pub fn get_byte_register(&self, register_index: usize) -> &u8 {
        let register = if cfg!(debug_assertions) {
            self.call_stack
                .last()
                .unwrap()
                .registers
                .bytes
                .get(register_index)
                .unwrap()
        } else {
            unsafe {
                self.call_stack
                    .last()
                    .unwrap_unchecked()
                    .registers
                    .bytes
                    .get_unchecked(register_index)
            }
        };

        match register {
            Register::Value(value) => value,
            Register::Pointer(pointer) => self.get_pointer_to_byte(pointer),
            Register::Empty => panic!("Attempted to get value from empty register"),
        }
    }

    pub fn get_pointer_to_byte(&self, pointer: &Pointer) -> &u8 {
        match pointer {
            Pointer::Register(register_index) => self.get_byte_register(*register_index),
            Pointer::Constant(constant_index) => {
                self.get_constant(*constant_index).as_byte().unwrap()
            }
            Pointer::Stack(call_index, register_index) => {
                let call_frame = if cfg!(debug_assertions) {
                    self.call_stack.get(*call_index).unwrap()
                } else {
                    unsafe { self.call_stack.get_unchecked(*call_index) }
                };
                let register = if cfg!(debug_assertions) {
                    call_frame.registers.bytes.get(*register_index).unwrap()
                } else {
                    unsafe { call_frame.registers.bytes.get_unchecked(*register_index) }
                };

                match register {
                    Register::Value(value) => value,
                    Register::Pointer(pointer) => self.get_pointer_to_byte(pointer),
                    Register::Empty => panic!("Attempted to get value from empty register"),
                }
            }
        }
    }

    pub fn set_byte_register(&mut self, register_index: usize, new_register: Register<u8>) {
        let old_register = if cfg!(debug_assertions) {
            self.call_stack
                .last_mut()
                .unwrap()
                .registers
                .bytes
                .get_mut(register_index)
                .unwrap()
        } else {
            unsafe {
                self.call_stack
                    .last_mut()
                    .unwrap_unchecked()
                    .registers
                    .bytes
                    .get_unchecked_mut(register_index)
            }
        };

        *old_register = new_register;
    }

    pub fn get_character_register(&self, register_index: usize) -> &char {
        let register = if cfg!(debug_assertions) {
            self.call_stack
                .last()
                .unwrap()
                .registers
                .characters
                .get(register_index)
                .unwrap()
        } else {
            unsafe {
                self.call_stack
                    .last()
                    .unwrap_unchecked()
                    .registers
                    .characters
                    .get_unchecked(register_index)
            }
        };

        match register {
            Register::Value(value) => value,
            Register::Pointer(pointer) => self.get_pointer_to_character(pointer),
            Register::Empty => panic!("Attempted to get value from empty register"),
        }
    }

    pub fn get_pointer_to_character(&self, pointer: &Pointer) -> &char {
        match pointer {
            Pointer::Register(register_index) => self.get_character_register(*register_index),
            Pointer::Constant(constant_index) => {
                self.get_constant(*constant_index).as_character().unwrap()
            }
            Pointer::Stack(call_index, register_index) => {
                let call_frame = if cfg!(debug_assertions) {
                    self.call_stack.get(*call_index).unwrap()
                } else {
                    unsafe { self.call_stack.get_unchecked(*call_index) }
                };
                let register = if cfg!(debug_assertions) {
                    call_frame
                        .registers
                        .characters
                        .get(*register_index)
                        .unwrap()
                } else {
                    unsafe {
                        call_frame
                            .registers
                            .characters
                            .get_unchecked(*register_index)
                    }
                };

                match register {
                    Register::Value(value) => value,
                    Register::Pointer(pointer) => self.get_pointer_to_character(pointer),
                    Register::Empty => panic!("Attempted to get value from empty register"),
                }
            }
        }
    }

    pub fn set_character_register(&mut self, register_index: usize, new_register: Register<char>) {
        let old_register = if cfg!(debug_assertions) {
            self.call_stack
                .last_mut()
                .unwrap()
                .registers
                .characters
                .get_mut(register_index)
                .unwrap()
        } else {
            unsafe {
                self.call_stack
                    .last_mut()
                    .unwrap_unchecked()
                    .registers
                    .characters
                    .get_unchecked_mut(register_index)
            }
        };

        *old_register = new_register;
    }

    pub fn get_float_register(&self, register_index: usize) -> &f64 {
        let register = if cfg!(debug_assertions) {
            self.call_stack
                .last()
                .unwrap()
                .registers
                .floats
                .get(register_index)
                .unwrap()
        } else {
            unsafe {
                self.call_stack
                    .last()
                    .unwrap_unchecked()
                    .registers
                    .floats
                    .get_unchecked(register_index)
            }
        };

        match register {
            Register::Value(value) => value,
            Register::Pointer(pointer) => self.get_pointer_to_float(pointer),
            Register::Empty => panic!("Attempted to get value from empty register"),
        }
    }

    pub fn get_pointer_to_float(&self, pointer: &Pointer) -> &f64 {
        match pointer {
            Pointer::Register(register_index) => self.get_float_register(*register_index),
            Pointer::Constant(constant_index) => {
                self.get_constant(*constant_index).as_float().unwrap()
            }
            Pointer::Stack(call_index, register_index) => {
                let call_frame = if cfg!(debug_assertions) {
                    self.call_stack.get(*call_index).unwrap()
                } else {
                    unsafe { self.call_stack.get_unchecked(*call_index) }
                };
                let register = if cfg!(debug_assertions) {
                    call_frame.registers.floats.get(*register_index).unwrap()
                } else {
                    unsafe { call_frame.registers.floats.get_unchecked(*register_index) }
                };

                match register {
                    Register::Value(value) => value,
                    Register::Pointer(pointer) => self.get_pointer_to_float(pointer),
                    Register::Empty => panic!("Attempted to get value from empty register"),
                }
            }
        }
    }

    pub fn set_float_register(&mut self, register_index: usize, new_register: Register<f64>) {
        let old_register = if cfg!(debug_assertions) {
            self.call_stack
                .last_mut()
                .unwrap()
                .registers
                .floats
                .get_mut(register_index)
                .unwrap()
        } else {
            unsafe {
                self.call_stack
                    .last_mut()
                    .unwrap_unchecked()
                    .registers
                    .floats
                    .get_unchecked_mut(register_index)
            }
        };

        *old_register = new_register;
    }

    pub fn get_integer_register(&self, register_index: usize) -> &i64 {
        let register = if cfg!(debug_assertions) {
            self.call_stack
                .last()
                .unwrap()
                .registers
                .integers
                .get(register_index)
                .unwrap()
        } else {
            unsafe {
                self.call_stack
                    .last()
                    .unwrap_unchecked()
                    .registers
                    .integers
                    .get_unchecked(register_index)
            }
        };

        match register {
            Register::Value(value) => value,
            Register::Pointer(pointer) => self.get_pointer_to_integer(pointer),
            Register::Empty => panic!("Attempted to get value from empty register"),
        }
    }

    pub fn get_pointer_to_integer(&self, pointer: &Pointer) -> &i64 {
        match pointer {
            Pointer::Register(register_index) => self.get_integer_register(*register_index),
            Pointer::Constant(constant_index) => {
                self.get_constant(*constant_index).as_integer().unwrap()
            }
            Pointer::Stack(call_index, register_index) => {
                let call_frame = if cfg!(debug_assertions) {
                    self.call_stack.get(*call_index).unwrap()
                } else {
                    unsafe { self.call_stack.get_unchecked(*call_index) }
                };
                let register = if cfg!(debug_assertions) {
                    call_frame.registers.integers.get(*register_index).unwrap()
                } else {
                    unsafe { call_frame.registers.integers.get_unchecked(*register_index) }
                };

                match register {
                    Register::Value(value) => value,
                    Register::Pointer(pointer) => self.get_pointer_to_integer(pointer),
                    Register::Empty => panic!("Attempted to get value from empty register"),
                }
            }
        }
    }

    pub fn set_integer_register(&mut self, register_index: usize, new_register: Register<i64>) {
        let old_register = if cfg!(debug_assertions) {
            self.call_stack
                .last_mut()
                .unwrap()
                .registers
                .integers
                .get_mut(register_index)
                .unwrap()
        } else {
            unsafe {
                self.call_stack
                    .last_mut()
                    .unwrap_unchecked()
                    .registers
                    .integers
                    .get_unchecked_mut(register_index)
            }
        };

        *old_register = new_register;
    }

    pub fn get_string_register(&self, register_index: usize) -> &DustString {
        let register = if cfg!(debug_assertions) {
            self.call_stack
                .last()
                .unwrap()
                .registers
                .strings
                .get(register_index)
                .unwrap()
        } else {
            unsafe {
                self.call_stack
                    .last()
                    .unwrap_unchecked()
                    .registers
                    .strings
                    .get_unchecked(register_index)
            }
        };

        match register {
            Register::Value(value) => value,
            Register::Pointer(pointer) => self.get_pointer_to_string(pointer),
            Register::Empty => panic!("Attempted to get value from empty register"),
        }
    }

    pub fn get_pointer_to_string(&self, pointer: &Pointer) -> &DustString {
        match pointer {
            Pointer::Register(register_index) => self.get_string_register(*register_index),
            Pointer::Constant(constant_index) => {
                self.get_constant(*constant_index).as_string().unwrap()
            }
            Pointer::Stack(call_index, register_index) => {
                let call_frame = if cfg!(debug_assertions) {
                    self.call_stack.get(*call_index).unwrap()
                } else {
                    unsafe { self.call_stack.get_unchecked(*call_index) }
                };
                let register = if cfg!(debug_assertions) {
                    call_frame.registers.strings.get(*register_index).unwrap()
                } else {
                    unsafe { call_frame.registers.strings.get_unchecked(*register_index) }
                };

                match register {
                    Register::Value(value) => value,
                    Register::Pointer(pointer) => self.get_pointer_to_string(pointer),
                    Register::Empty => panic!("Attempted to get value from empty register"),
                }
            }
        }
    }

    pub fn set_string_register(
        &mut self,
        register_index: usize,
        new_register: Register<DustString>,
    ) {
        let old_register = if cfg!(debug_assertions) {
            self.call_stack
                .last_mut()
                .unwrap()
                .registers
                .strings
                .get_mut(register_index)
                .unwrap()
        } else {
            unsafe {
                self.call_stack
                    .last_mut()
                    .unwrap_unchecked()
                    .registers
                    .strings
                    .get_unchecked_mut(register_index)
            }
        };

        *old_register = new_register;
    }

    pub fn get_constant(&self, constant_index: usize) -> &ConcreteValue {
        if cfg!(debug_assertions) {
            self.chunk.constants.get(constant_index).unwrap()
        } else {
            unsafe { self.chunk.constants.get_unchecked(constant_index) }
        }
    }
}
