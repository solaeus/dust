use std::{sync::Arc, thread::JoinHandle};

use tracing::{info, span};

use crate::{
    Chunk, DustString, Span, Value,
    vm::{CallFrame, action::ActionSequence},
};

use super::{Register, call_frame::Pointer};

pub struct Thread {
    chunk: Arc<Chunk>,
    call_stack: Vec<CallFrame>,
    return_value: Option<Option<Value>>,
    spawned_threads: Vec<JoinHandle<()>>,
}

impl Thread {
    pub fn new(chunk: Arc<Chunk>) -> Self {
        let mut call_stack = Vec::with_capacity(chunk.prototypes.len() + 1);
        let mut main_call = CallFrame::new(chunk.clone(), 0);
        main_call.ip = 1; // The first action is already known

        call_stack.push(main_call);

        Thread {
            chunk,
            call_stack,
            return_value: None,
            spawned_threads: Vec::new(),
        }
    }

    pub fn run(mut self) -> Option<Value> {
        let span = span!(tracing::Level::INFO, "Thread");
        let _ = span.enter();

        info!(
            "Starting thread with {}",
            self.chunk
                .name
                .clone()
                .unwrap_or_else(|| DustString::from("anonymous"))
        );

        let actions = ActionSequence::new(&self.chunk.instructions);

        loop {
            let ip = self.current_frame().ip;
            let next_action = if cfg!(debug_assertions) {
                actions.actions.get(ip).unwrap()
            } else {
                unsafe { actions.actions.get_unchecked(ip) }
            };

            (next_action.logic)(next_action.instruction, &mut self);

            if let Some(return_value) = self.return_value {
                return return_value;
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

    pub fn get_frame(&self, index: usize) -> &CallFrame {
        if cfg!(debug_assertions) {
            self.call_stack.get(index).unwrap()
        } else {
            unsafe { self.call_stack.get_unchecked(index) }
        }
    }

    pub fn get_boolean_register(&self, index: usize) -> bool {
        let register = if cfg!(debug_assertions) {
            self.current_frame()
                .registers
                .booleans
                .get(index as usize)
                .unwrap()
        } else {
            unsafe {
                self.current_frame()
                    .registers
                    .booleans
                    .get_unchecked(index as usize)
            }
        };

        match register {
            Register::Value(boolean) => *boolean,
            Register::Pointer(pointer) => self.follow_pointer_to_boolean(*pointer),
            Register::Empty => panic!("Attempted to get a boolean from an empty register"),
        }
    }

    pub fn set_boolean_register(&mut self, index: usize, new_register: Register<bool>) {
        let old_register = if cfg!(debug_assertions) {
            self.current_frame_mut()
                .registers
                .booleans
                .get_mut(index as usize)
                .unwrap()
        } else {
            unsafe {
                self.current_frame_mut()
                    .registers
                    .booleans
                    .get_unchecked_mut(index as usize)
            }
        };

        *old_register = new_register;
    }

    pub fn get_byte_register(&self, index: usize) -> u8 {
        let register = if cfg!(debug_assertions) {
            self.current_frame()
                .registers
                .bytes
                .get(index as usize)
                .unwrap()
        } else {
            unsafe {
                self.current_frame()
                    .registers
                    .bytes
                    .get_unchecked(index as usize)
            }
        };

        match register {
            Register::Value(byte) => *byte,
            Register::Pointer(pointer) => self.follow_pointer_to_byte(*pointer),
            Register::Empty => panic!("Attempted to get a byte from an empty register"),
        }
    }

    pub fn set_byte_register(&mut self, index: usize, new_register: Register<u8>) {
        let old_register = if cfg!(debug_assertions) {
            self.current_frame_mut()
                .registers
                .bytes
                .get_mut(index as usize)
                .unwrap()
        } else {
            unsafe {
                self.current_frame_mut()
                    .registers
                    .bytes
                    .get_unchecked_mut(index as usize)
            }
        };

        *old_register = new_register;
    }

    pub fn get_character_register(&self, index: usize) -> char {
        let register = if cfg!(debug_assertions) {
            self.current_frame()
                .registers
                .characters
                .get(index as usize)
                .unwrap()
        } else {
            unsafe {
                self.current_frame()
                    .registers
                    .characters
                    .get_unchecked(index as usize)
            }
        };

        match register {
            Register::Value(character) => *character,
            Register::Pointer(pointer) => self.follow_pointer_to_character(*pointer),
            Register::Empty => panic!("Attempted to get a character from an empty register"),
        }
    }

    pub fn set_character_register(&mut self, index: usize, new_register: Register<char>) {
        let old_register = if cfg!(debug_assertions) {
            self.current_frame_mut()
                .registers
                .characters
                .get_mut(index as usize)
                .unwrap()
        } else {
            unsafe {
                self.current_frame_mut()
                    .registers
                    .characters
                    .get_unchecked_mut(index as usize)
            }
        };

        *old_register = new_register;
    }

    pub fn get_float_register(&self, index: usize) -> f64 {
        let register = if cfg!(debug_assertions) {
            self.current_frame()
                .registers
                .floats
                .get(index as usize)
                .unwrap()
        } else {
            unsafe {
                self.current_frame()
                    .registers
                    .floats
                    .get_unchecked(index as usize)
            }
        };

        match register {
            Register::Value(float) => *float,
            Register::Pointer(pointer) => self.follow_pointer_to_float(*pointer),
            Register::Empty => panic!("Attempted to get a float from an empty register"),
        }
    }

    pub fn set_float_register(&mut self, index: usize, new_register: Register<f64>) {
        let old_register = if cfg!(debug_assertions) {
            self.current_frame_mut()
                .registers
                .floats
                .get_mut(index as usize)
                .unwrap()
        } else {
            unsafe {
                self.current_frame_mut()
                    .registers
                    .floats
                    .get_unchecked_mut(index as usize)
            }
        };

        *old_register = new_register;
    }

    pub fn get_integer_register(&self, index: usize) -> i64 {
        let register = if cfg!(debug_assertions) {
            self.current_frame()
                .registers
                .integers
                .get(index as usize)
                .unwrap()
        } else {
            unsafe {
                self.current_frame()
                    .registers
                    .integers
                    .get_unchecked(index as usize)
            }
        };

        match register {
            Register::Value(integer) => *integer,
            Register::Pointer(pointer) => self.follow_pointer_to_integer(*pointer),
            Register::Empty => panic!("Attempted to get an integer from an empty register"),
        }
    }

    pub fn set_integer_register(&mut self, index: usize, new_register: Register<i64>) {
        let old_register = if cfg!(debug_assertions) {
            self.current_frame_mut()
                .registers
                .integers
                .get_mut(index as usize)
                .unwrap()
        } else {
            unsafe {
                self.current_frame_mut()
                    .registers
                    .integers
                    .get_unchecked_mut(index as usize)
            }
        };

        *old_register = new_register;
    }

    pub fn get_string_register(&self, index: usize) -> &DustString {
        let register = if cfg!(debug_assertions) {
            self.current_frame()
                .registers
                .strings
                .get(index as usize)
                .unwrap()
        } else {
            unsafe {
                self.current_frame()
                    .registers
                    .strings
                    .get_unchecked(index as usize)
            }
        };

        match register {
            Register::Value(string) => string,
            Register::Pointer(pointer) => self.follow_pointer_to_string(*pointer),
            Register::Empty => panic!("Attempted to get a string from an empty register"),
        }
    }

    pub fn set_string_register(&mut self, index: usize, new_register: Register<DustString>) {
        let old_register = if cfg!(debug_assertions) {
            self.current_frame_mut()
                .registers
                .strings
                .get_mut(index as usize)
                .unwrap()
        } else {
            unsafe {
                self.current_frame_mut()
                    .registers
                    .strings
                    .get_unchecked_mut(index as usize)
            }
        };

        *old_register = new_register;
    }

    pub fn follow_pointer_to_boolean(&self, pointer: Pointer) -> bool {
        match pointer {
            Pointer::Register(register_index) => self.get_boolean_register(register_index),
            Pointer::Constant(_) => {
                panic!("Attempted to access boolean from a constant pointer")
            }
        }
    }

    pub fn follow_pointer_to_byte(&self, pointer: Pointer) -> u8 {
        match pointer {
            Pointer::Register(register_index) => self.get_byte_register(register_index),
            Pointer::Constant(_) => {
                panic!("Attempted to access byte from a constant pointer")
            }
        }
    }

    pub fn follow_pointer_to_character(&self, pointer: Pointer) -> char {
        match pointer {
            Pointer::Register(register_index) => self.get_character_register(register_index),
            Pointer::Constant(constant_index) => {
                if cfg!(debug_assertions) {
                    *self
                        .chunk
                        .constant_characters
                        .get(constant_index as usize)
                        .unwrap()
                } else {
                    unsafe {
                        *self
                            .chunk
                            .constant_characters
                            .get_unchecked(constant_index as usize)
                    }
                }
            }
        }
    }

    pub fn follow_pointer_to_float(&self, pointer: Pointer) -> f64 {
        match pointer {
            Pointer::Register(register_index) => self.get_float_register(register_index),
            Pointer::Constant(constant_index) => {
                if cfg!(debug_assertions) {
                    *self
                        .chunk
                        .constant_floats
                        .get(constant_index as usize)
                        .unwrap()
                } else {
                    unsafe {
                        *self
                            .chunk
                            .constant_floats
                            .get_unchecked(constant_index as usize)
                    }
                }
            }
        }
    }

    pub fn follow_pointer_to_integer(&self, pointer: Pointer) -> i64 {
        match pointer {
            Pointer::Register(register_index) => self.get_integer_register(register_index),
            Pointer::Constant(constant_index) => {
                if cfg!(debug_assertions) {
                    *self
                        .chunk
                        .constant_integers
                        .get(constant_index as usize)
                        .unwrap()
                } else {
                    unsafe {
                        *self
                            .chunk
                            .constant_integers
                            .get_unchecked(constant_index as usize)
                    }
                }
            }
        }
    }

    pub fn follow_pointer_to_string(&self, pointer: Pointer) -> &DustString {
        match pointer {
            Pointer::Register(register_index) => self.get_string_register(register_index),
            Pointer::Constant(constant_index) => {
                if cfg!(debug_assertions) {
                    self.chunk
                        .constant_strings
                        .get(constant_index as usize)
                        .unwrap()
                } else {
                    unsafe {
                        self.chunk
                            .constant_strings
                            .get_unchecked(constant_index as usize)
                    }
                }
            }
        }
    }
}
