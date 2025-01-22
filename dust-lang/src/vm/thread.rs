use std::{sync::Arc, thread::JoinHandle};

use smallvec::SmallVec;
use tracing::{info, span, trace};

use crate::{
    Chunk, DustString, TypeCode, Value,
    vm::{
        Register,
        action::{ActionData, ActionSequence},
    },
};

use super::{Action, CallFrame};

pub struct Thread {
    chunk: Arc<Chunk>,
}

impl Thread {
    pub fn new(chunk: Arc<Chunk>) -> Self {
        Thread { chunk }
    }

    pub fn run(&mut self) -> Option<Value> {
        let span = span!(tracing::Level::INFO, "Thread");
        let _enter = span.enter();

        info!(
            "Starting thread with {}",
            self.chunk
                .name
                .clone()
                .unwrap_or_else(|| DustString::from("anonymous"))
        );

        let mut call_stack = SmallVec::with_capacity(self.chunk.prototypes.len() + 1);
        let main_call = CallFrame::new(Arc::clone(&self.chunk), 0);

        call_stack.push(main_call);

        let mut thread_data = ThreadData {
            stack: call_stack,
            return_value: None,
            spawned_threads: Vec::new(),
        };
        let mut action_sequence = ActionSequence::new(&self.chunk.instructions);

        trace!("Run thread main with actions: {:?}", action_sequence);

        loop {
            let current_frame = thread_data.current_frame_mut();
            let current_action = action_sequence.get_mut(current_frame.ip);
            current_frame.ip += 1;

            trace!("Action: {}", current_action);

            // (current_action.logic)(&mut thread_data, &mut current_action.data);

            match current_action.data.name {
                "LOAD_CONSTANT" => {
                    let ActionData {
                        instruction,
                        integer_pointers,
                        integer_register_pointers,
                        runs,
                        ..
                    } = &mut current_action.data;
                    let r#type = instruction.b_type;

                    if *runs > 0 {
                        match r#type {
                            TypeCode::INTEGER => unsafe {
                                *integer_register_pointers[0] =
                                    Register::Value(*integer_pointers[0]);
                            },
                            unknown => unknown.panic_from_unknown_code(),
                        }

                        continue;
                    }

                    let current_frame = thread_data.current_frame_mut();
                    let destination = instruction.a_field;
                    let constant_index = instruction.b_field;

                    match r#type {
                        TypeCode::INTEGER => {
                            let mut value = *current_frame
                                .prototype
                                .constants
                                .get_integer(constant_index)
                                .unwrap();
                            let new_register = Register::Value(value);
                            let old_register = current_frame.registers.get_integer_mut(destination);

                            *old_register = new_register;
                            integer_pointers[0] = &mut value;
                            integer_register_pointers[0] = old_register;
                        }
                        unknown => unknown.panic_from_unknown_code(),
                    };

                    if instruction.c_field != 0 {
                        current_frame.ip += 1;
                    }

                    *runs += 1;
                }
                "LESS_INT" => {
                    let ActionData {
                        instruction,
                        integer_pointers,
                        integer_register_pointers,
                        runs,
                        ..
                    } = &mut current_action.data;
                    let r#type = instruction.b_type;

                    if *runs > 0 {
                        unsafe {
                            // if *pointers[0] < *pointers[1] {
                            //     current_frame.ip += 1;
                            // }
                        }
                    } else {
                        let (is_less, left_pointer, right_pointer) =
                            match (instruction.b_is_constant, instruction.c_is_constant) {
                                (true, true) => {
                                    let left = current_frame
                                        .prototype
                                        .constants
                                        .get_integer(instruction.b_field)
                                        .unwrap();
                                    let right = current_frame
                                        .prototype
                                        .constants
                                        .get_integer(instruction.c_field)
                                        .unwrap();
                                    let is_less = left < right;

                                    (
                                        is_less,
                                        Box::into_raw(Box::new(*left)),
                                        Box::into_raw(Box::new(*right)),
                                    )
                                }
                                (true, false) => {
                                    let left = *current_frame
                                        .prototype
                                        .constants
                                        .get_integer(instruction.b_field)
                                        .unwrap();
                                    let right = current_frame
                                        .registers
                                        .get_integer_mut(instruction.c_field)
                                        .expect_value_mut();
                                    let is_less = left < *right;

                                    (is_less, Box::into_raw(Box::new(left)), right as *mut i64)
                                }
                                (false, true) => {
                                    let right = *current_frame
                                        .prototype
                                        .constants
                                        .get_integer(instruction.c_field)
                                        .unwrap();
                                    let left = current_frame
                                        .registers
                                        .get_integer_mut(instruction.b_field)
                                        .expect_value_mut();
                                    let is_less = *left < right;

                                    (is_less, left as *mut i64, Box::into_raw(Box::new(right)))
                                }
                                (false, false) => {
                                    let [left, right] =
                                        current_frame.registers.get_many_integer_mut([
                                            instruction.b_field as usize,
                                            instruction.c_field as usize,
                                        ]);
                                    let left = left.expect_value_mut();
                                    let right = right.expect_value_mut();
                                    let is_less = *left < *right;

                                    (is_less, left as *mut i64, right as *mut i64)
                                }
                            };

                        if is_less {
                            current_frame.ip += 1;
                        }

                        integer_pointers[0] = left_pointer;
                        integer_pointers[1] = right_pointer;
                        *runs += 1;
                    }
                }
                _ => todo!(),
            }

            if let Some(value_option) = thread_data.return_value {
                return value_option;
            }
        }
    }
}

#[derive(Debug)]
pub struct ThreadData {
    pub stack: SmallVec<[CallFrame; 10]>,
    pub return_value: Option<Option<Value>>,
    pub spawned_threads: Vec<JoinHandle<()>>,
}

impl ThreadData {
    pub fn current_frame(&self) -> &CallFrame {
        if cfg!(debug_assertions) {
            self.stack.last().unwrap()
        } else {
            unsafe { self.stack.last().unwrap_unchecked() }
        }
    }

    pub fn current_frame_mut(&mut self) -> &mut CallFrame {
        if cfg!(debug_assertions) {
            self.stack.last_mut().unwrap()
        } else {
            unsafe { self.stack.last_mut().unwrap_unchecked() }
        }
    }
}
