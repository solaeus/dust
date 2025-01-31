use std::{sync::Arc, thread::JoinHandle};

use tracing::{info, trace};

use crate::{Chunk, DustString, Span, Value, vm::CallFrame};

use super::{Action, Register};

pub struct Thread {
    chunk: Arc<Chunk>,
    call_stack: Vec<CallFrame>,
    next_action: Action,
    return_value: Option<Value>,
    spawned_threads: Vec<JoinHandle<()>>,
}

impl Thread {
    pub fn new(chunk: Arc<Chunk>) -> Self {
        let mut call_stack = Vec::with_capacity(chunk.prototypes.len() + 1);
        let mut main_call = CallFrame::new(chunk.clone(), 0);
        main_call.ip = 1; // The first action is already known

        call_stack.push(main_call);

        let first_action = Action::from(*chunk.instructions.first().unwrap());

        Thread {
            chunk,
            call_stack,
            next_action: first_action,
            return_value: None,
            spawned_threads: Vec::new(),
        }
    }

    pub fn run(&mut self) -> Option<Value> {
        info!(
            "Starting thread with {}",
            self.chunk
                .name
                .clone()
                .unwrap_or_else(|| DustString::from("anonymous"))
        );

        loop {
            trace!("Instruction: {}", self.next_action.instruction);

            let should_end = (self.next_action.logic)(self.next_action.instruction, self);

            if should_end {
                self.spawned_threads.into_iter().for_each(|join_handle| {
                    let _ = join_handle.join();
                });

                return self.return_value.take();
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
            Register::Pointer(pointer) => unsafe { **pointer },
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
            Register::Pointer(pointer) => unsafe { **pointer },
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
            Register::Pointer(pointer) => unsafe { **pointer },
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
            Register::Pointer(pointer) => unsafe { **pointer },
            Register::Empty => panic!("Attempted to get a float from an empty register"),
        }
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
            Register::Pointer(pointer) => unsafe { **pointer },
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
            Register::Pointer(pointer) => {
                if cfg!(debug_assertions) {
                    unsafe { pointer.as_ref().unwrap() }
                } else {
                    unsafe { pointer.as_ref().unwrap_unchecked() }
                }
            }
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
}
