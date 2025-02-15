mod add;
mod jump;
mod less;

use add::add_integers;
use jump::jump;
use less::less_integers;

use tracing::trace;

use std::{
    fmt::{self, Display, Formatter},
    ptr,
};

use crate::{
    instruction::{InstructionFields, TypeCode},
    AbstractList, ConcreteValue, Operation, Value,
};

use super::{thread::Thread, Pointer, Register};

#[derive(Clone, Debug)]
pub struct ActionSequence {
    pub actions: Vec<(Action, InstructionFields)>,
}

impl ActionSequence {
    #[allow(clippy::while_let_on_iterator)]
    pub fn new(instructions: Vec<InstructionFields>) -> Self {
        let mut actions = Vec::with_capacity(instructions.len());
        let mut instructions_reversed = instructions.into_iter().rev();

        while let Some(instruction) = instructions_reversed.next() {
            if instruction.operation == Operation::JUMP {
                let backward_offset = instruction.b_field as usize;
                let is_positive = instruction.c_field != 0;

                if !is_positive {
                    let mut loop_actions = Vec::with_capacity(backward_offset + 1);
                    let jump_action = Action::optimized(&instruction);

                    loop_actions.push((jump_action, instruction));

                    for _ in 0..backward_offset {
                        let instruction = instructions_reversed.next().unwrap();
                        let action = Action::optimized(&instruction);

                        loop_actions.push((action, instruction));
                    }

                    loop_actions.reverse();

                    let r#loop = Action::r#loop(ActionSequence {
                        actions: loop_actions,
                    });

                    actions.push((r#loop, instruction));

                    continue;
                }
            }

            let action = Action::unoptimized(instruction);

            actions.push((action, instruction));
        }

        actions.reverse();

        ActionSequence { actions }
    }

    pub fn len(&self) -> usize {
        self.actions.len()
    }

    pub fn run(&mut self, thread: &mut Thread) {
        let mut local_ip = 0;

        while local_ip < self.actions.len() {
            let (action, instruction) = &mut self.actions[local_ip];
            local_ip += 1;

            trace!("Run {action}");

            match action {
                Action::Unoptimized { logic, instruction } => {
                    logic(&mut local_ip, &*instruction, thread);
                }
                Action::Loop { actions } => {
                    actions.run(thread);
                }
                Action::OptimizedAddIntegers {
                    destination_pointer,
                    left_pointer,
                    right_pointer,
                } => {
                    let left = if left_pointer.is_null() || !left_pointer.is_aligned() {
                        let left_index = instruction.b_field as usize;
                        let left_is_constant = instruction.b_is_constant;
                        let left_value = thread.get_integer(left_index, left_is_constant);

                        *left_pointer = left_value;

                        left_value
                    } else {
                        unsafe { &ptr::read(*left_pointer) }
                    };
                    let right = if right_pointer.is_null() || !right_pointer.is_aligned() {
                        let right_index = instruction.c_field as usize;
                        let right_is_constant = instruction.c_is_constant;
                        let right_value = thread.get_integer(right_index, right_is_constant);

                        *right_pointer = right_value;

                        right_value
                    } else {
                        unsafe { &ptr::read(*right_pointer) }
                    };
                    let sum = left.saturating_add(*right);

                    if destination_pointer.is_null() || !destination_pointer.is_aligned() {
                        let destination = instruction.a_field as usize;
                        let register = Register::Value(sum);

                        thread.set_integer_register(destination, register);
                    } else {
                        unsafe {
                            **destination_pointer = sum;
                        }
                    }
                }
                Action::OptimizedLessIntegers {
                    left_pointer,
                    right_pointer,
                } => {
                    let left = if left_pointer.is_null() || !left_pointer.is_aligned() {
                        let left_index = instruction.b_field as usize;
                        let left_is_constant = instruction.b_is_constant;
                        let left_value = thread.get_integer(left_index, left_is_constant);

                        *left_pointer = left_value;

                        left_value
                    } else {
                        unsafe { &ptr::read(*left_pointer) }
                    };
                    let right = if right_pointer.is_null() || !right_pointer.is_aligned() {
                        let right_index = instruction.c_field as usize;
                        let right_is_constant = instruction.c_is_constant;
                        let right_value = thread.get_integer(right_index, right_is_constant);

                        *right_pointer = right_value;

                        right_value
                    } else {
                        unsafe { &ptr::read(*right_pointer) }
                    };
                    let is_less_than = left < right;
                    let comparator = instruction.d_field;

                    if is_less_than == comparator {
                        local_ip += 1;
                    }
                }
                Action::OptimizedJumpForward { offset } => {
                    local_ip += *offset;
                }
                Action::OptimizedJumpBackward { offset } => {
                    local_ip -= *offset + 1;
                }
            }
        }
    }
}

impl Display for ActionSequence {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "[")?;

        for (index, (action, _)) in self.actions.iter().enumerate() {
            if index > 0 {
                write!(f, ", ")?;
            }

            write!(f, "{action}")?;
        }

        write!(f, "]")
    }
}

#[derive(Clone, Debug)]
pub enum Action {
    Unoptimized {
        logic: ActionLogic,
        instruction: InstructionFields,
    },
    Loop {
        actions: ActionSequence,
    },
    OptimizedAddIntegers {
        destination_pointer: *mut i64,
        left_pointer: *const i64,
        right_pointer: *const i64,
    },
    OptimizedLessIntegers {
        left_pointer: *const i64,
        right_pointer: *const i64,
    },
    OptimizedJumpForward {
        offset: usize,
    },
    OptimizedJumpBackward {
        offset: usize,
    },
}

impl Action {
    pub fn unoptimized(instruction: InstructionFields) -> Self {
        let logic = match instruction.operation {
            Operation::POINT => point,
            Operation::CLOSE => close,
            Operation::LOAD_ENCODED => load_encoded,
            Operation::LOAD_CONSTANT => load_constant,
            Operation::LOAD_LIST => load_list,
            Operation::LOAD_FUNCTION => load_function,
            Operation::LOAD_SELF => load_self,
            Operation::ADD => match (instruction.b_type, instruction.c_type) {
                (TypeCode::INTEGER, TypeCode::INTEGER) => add_integers,
                _ => todo!(),
            },
            Operation::SUBTRACT => subtract,
            Operation::MULTIPLY => multiply,
            Operation::DIVIDE => divide,
            Operation::MODULO => modulo,
            Operation::NEGATE => negate,
            Operation::NOT => not,
            Operation::EQUAL => equal,
            Operation::LESS => match (instruction.b_type, instruction.c_type) {
                (TypeCode::INTEGER, TypeCode::INTEGER) => less_integers,
                _ => todo!(),
            },
            Operation::LESS_EQUAL => less_equal,
            Operation::TEST => test,
            Operation::TEST_SET => test_set,
            Operation::CALL => call,
            Operation::CALL_NATIVE => call_native,
            Operation::JUMP => jump,
            Operation::RETURN => r#return,
            _ => todo!(),
        };

        Action::Unoptimized { logic, instruction }
    }

    pub fn optimized(instruction: &InstructionFields) -> Self {
        match instruction.operation {
            Operation::ADD => match (instruction.b_type, instruction.c_type) {
                (TypeCode::INTEGER, TypeCode::INTEGER) => Action::OptimizedAddIntegers {
                    destination_pointer: std::ptr::null_mut(),
                    left_pointer: std::ptr::null(),
                    right_pointer: std::ptr::null(),
                },
                _ => todo!(),
            },
            Operation::LESS => match (instruction.b_type, instruction.c_type) {
                (TypeCode::INTEGER, TypeCode::INTEGER) => Action::OptimizedLessIntegers {
                    left_pointer: std::ptr::null(),
                    right_pointer: std::ptr::null(),
                },
                _ => todo!(),
            },
            Operation::JUMP => {
                let offset = instruction.b_field as usize;
                let is_positive = instruction.c_field != 0;

                if is_positive {
                    Action::OptimizedJumpForward { offset }
                } else {
                    Action::OptimizedJumpBackward { offset }
                }
            }
            _ => todo!(),
        }
    }

    pub fn r#loop(actions: ActionSequence) -> Self {
        Action::Loop { actions }
    }
}

impl Display for Action {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Action::Unoptimized { instruction, .. } => {
                write!(f, "{}", instruction.operation)
            }
            Action::Loop { actions } => {
                write!(f, "LOOP: {actions}")
            }
            Action::OptimizedAddIntegers { .. } => {
                write!(f, "ADD integers optimized")
            }
            Action::OptimizedLessIntegers { .. } => {
                write!(f, "LESS integers optimized")
            }
            Action::OptimizedJumpForward { offset } => {
                write!(f, "JUMP +{offset}")
            }
            Action::OptimizedJumpBackward { offset } => {
                write!(f, "JUMP -{offset}")
            }
        }
    }
}

pub type ActionLogic = fn(&mut usize, &InstructionFields, &mut Thread);

fn point(_: &mut usize, instruction: &InstructionFields, thread: &mut Thread) {
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

fn close(_: &mut usize, instruction: &InstructionFields, thread: &mut Thread) {
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

fn load_encoded(ip: &mut usize, instruction: &InstructionFields, thread: &mut Thread) {
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
        *ip += 1;
    }
}

fn load_constant(ip: &mut usize, instruction: &InstructionFields, thread: &mut Thread) {
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
        *ip += 1;
    }
}

fn load_list(ip: &mut usize, instruction: &InstructionFields, thread: &mut Thread) {
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
        *ip += 1;
    }
}

fn load_function(_: &mut usize, _: &InstructionFields, _: &mut Thread) {
    todo!()
}

fn load_self(_: &mut usize, _: &InstructionFields, _: &mut Thread) {
    todo!()
}

fn subtract(_: &mut usize, instruction: &InstructionFields, thread: &mut Thread) {
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

fn multiply(_: &mut usize, instruction: &InstructionFields, thread: &mut Thread) {
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

fn divide(_: &mut usize, instruction: &InstructionFields, thread: &mut Thread) {
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

fn modulo(_: &mut usize, instruction: &InstructionFields, thread: &mut Thread) {
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

fn test(ip: &mut usize, instruction: &InstructionFields, thread: &mut Thread) {
    let operand_register = instruction.b_field as usize;
    let test_value = instruction.c_field != 0;
    let operand_boolean = thread.get_boolean_register(operand_register);

    if *operand_boolean == test_value {
        *ip += 1;
    }
}

fn test_set(_: &mut usize, _: &InstructionFields, _: &mut Thread) {
    todo!()
}

fn equal(ip: &mut usize, instruction: &InstructionFields, thread: &mut Thread) {
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
                *ip += 1;
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
                *ip += 1;
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
                *ip += 1;
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
                *ip += 1;
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
                *ip += 1;
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
                *ip += 1;
            }
        }
        _ => unreachable!(),
    }
}

fn less_equal(ip: &mut usize, instruction: &InstructionFields, thread: &mut Thread) {
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
                *ip += 1;
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
                *ip += 1;
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
                *ip += 1;
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
                *ip += 1;
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
                *ip += 1;
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
                *ip += 1;
            }
        }
        _ => unreachable!(),
    }
}

fn negate(_: &mut usize, _: &InstructionFields, _: &mut Thread) {
    todo!()
}

fn not(_: &mut usize, _: &InstructionFields, _: &mut Thread) {
    todo!()
}

fn call(_: &mut usize, _: &InstructionFields, _: &mut Thread) {
    todo!()
}

fn call_native(_: &mut usize, _: &InstructionFields, _: &mut Thread) {
    todo!()
}

fn r#return(_: &mut usize, instruction: &InstructionFields, thread: &mut Thread) {
    let should_return_value = instruction.b_field != 0;
    let return_register = instruction.c_field as usize;
    let return_type = instruction.b_type;

    if should_return_value {
        match return_type {
            TypeCode::BOOLEAN => {
                let return_value = *thread.get_boolean_register(return_register);
                thread.return_value = Some(Value::boolean(return_value));
            }
            TypeCode::BYTE => {
                let return_value = *thread.get_byte_register(return_register);
                thread.return_value = Some(Value::byte(return_value));
            }
            TypeCode::CHARACTER => {
                let return_value = *thread.get_character_register(return_register);
                thread.return_value = Some(Value::character(return_value));
            }
            TypeCode::FLOAT => {
                let return_value = *thread.get_float_register(return_register);
                thread.return_value = Some(Value::float(return_value));
            }
            TypeCode::INTEGER => {
                let return_value = *thread.get_integer_register(return_register);
                thread.return_value = Some(Value::integer(return_value));
            }
            TypeCode::STRING => {
                let return_value = thread.get_string_register(return_register).clone();
                thread.return_value = Some(Value::string(return_value));
            }
            TypeCode::LIST => {
                let abstract_list = thread.get_list_register(return_register).clone();
                let mut items = Vec::with_capacity(abstract_list.item_pointers.len());

                for pointer in &abstract_list.item_pointers {
                    let value = thread.get_value_from_pointer(pointer);

                    items.push(value);
                }

                thread.return_value = Some(Value::Concrete(ConcreteValue::List {
                    items,
                    item_type: abstract_list.item_type,
                }));
            }
            _ => unreachable!(),
        }
    }
}
