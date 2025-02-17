mod add;
mod jump;
mod less;

use add::{
    add_bytes, add_character_string, add_characters, add_floats, add_integers,
    add_string_character, add_strings, optimized_add_integer,
};
use jump::jump;
use less::{
    less_booleans, less_bytes, less_characters, less_floats, less_integers, less_strings,
    optimized_less_integers,
};

use tracing::info;

use std::fmt::{self, Display, Formatter};

use crate::{
    instruction::{InstructionFields, TypeCode},
    Operation, Value,
};

use super::{call_frame::RuntimeValue, thread::Thread};

#[derive(Debug)]
pub struct ActionSequence {
    actions: Vec<(Action, InstructionFields)>,
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

    pub fn run(&mut self, thread: &mut Thread) {
        let mut local_ip = 0;

        while local_ip < self.actions.len() {
            let (action, instruction) = &mut self.actions[local_ip];
            local_ip += 1;

            info!("Run {action}");

            match action {
                Action::Unoptimized { logic, instruction } => {
                    logic(&mut local_ip, &*instruction, thread);
                }
                Action::Loop { actions } => {
                    actions.run(thread);
                }
                Action::OptimizedAddIntegers(cache) => {
                    optimized_add_integer(instruction, thread, cache);
                }
                Action::OptimizedLessIntegers(cache) => {
                    optimized_less_integers(&mut local_ip, instruction, thread, cache);
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

#[derive(Debug)]
enum Action {
    Unoptimized {
        logic: ActionLogic,
        instruction: InstructionFields,
    },
    Loop {
        actions: ActionSequence,
    },
    OptimizedAddIntegers(Option<[RuntimeValue<i64>; 3]>),
    OptimizedLessIntegers(Option<[RuntimeValue<i64>; 2]>),
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
                (TypeCode::FLOAT, TypeCode::FLOAT) => add_floats,
                (TypeCode::BYTE, TypeCode::BYTE) => add_bytes,
                (TypeCode::STRING, TypeCode::STRING) => add_strings,
                (TypeCode::CHARACTER, TypeCode::CHARACTER) => add_characters,
                (TypeCode::STRING, TypeCode::CHARACTER) => add_string_character,
                (TypeCode::CHARACTER, TypeCode::STRING) => add_character_string,
                _ => unreachable!(),
            },
            Operation::SUBTRACT => subtract,
            Operation::MULTIPLY => multiply,
            Operation::DIVIDE => divide,
            Operation::MODULO => modulo,
            Operation::NEGATE => negate,
            Operation::NOT => not,
            Operation::EQUAL => equal,
            Operation::LESS => match (instruction.b_type, instruction.c_type) {
                (TypeCode::BOOLEAN, TypeCode::BOOLEAN) => less_booleans,
                (TypeCode::BYTE, TypeCode::BYTE) => less_bytes,
                (TypeCode::CHARACTER, TypeCode::CHARACTER) => less_characters,
                (TypeCode::FLOAT, TypeCode::FLOAT) => less_floats,
                (TypeCode::INTEGER, TypeCode::INTEGER) => less_integers,
                (TypeCode::STRING, TypeCode::STRING) => less_strings,
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
                (TypeCode::INTEGER, TypeCode::INTEGER) => Action::OptimizedAddIntegers(None),
                _ => todo!(),
            },
            Operation::LESS => match (instruction.b_type, instruction.c_type) {
                (TypeCode::INTEGER, TypeCode::INTEGER) => Action::OptimizedLessIntegers(None),
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
                write!(f, "ADD_INTEGERS_OPTIMIZED")
            }
            Action::OptimizedLessIntegers { .. } => {
                write!(f, "LESS_INTEGERS_OPTIMIZED")
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
    todo!()
}

fn close(_: &mut usize, instruction: &InstructionFields, thread: &mut Thread) {
    todo!()
}

fn load_encoded(ip: &mut usize, instruction: &InstructionFields, thread: &mut Thread) {
    todo!()
}

fn load_constant(ip: &mut usize, instruction: &InstructionFields, thread: &mut Thread) {
    let destination = instruction.a_field as usize;
    let constant_index = instruction.b_field as usize;
    let constant_type = instruction.b_type;
    let jump_next = instruction.c_field != 0;
    let current_frame = thread.current_frame_mut();

    match constant_type {
        TypeCode::CHARACTER => {
            let constant = current_frame.get_character_constant(constant_index).clone();

            current_frame
                .registers
                .characters
                .get_mut(destination)
                .set(constant);
        }
        TypeCode::FLOAT => {
            let constant = current_frame.get_float_constant(constant_index).clone();

            current_frame
                .registers
                .floats
                .get_mut(destination)
                .set(constant);
        }
        TypeCode::INTEGER => {
            let constant = current_frame.get_integer_constant(constant_index).clone();

            current_frame
                .registers
                .integers
                .get_mut(destination)
                .set(constant);
        }
        TypeCode::STRING => {
            let constant = current_frame.get_string_constant(constant_index).clone();

            current_frame
                .registers
                .strings
                .get_mut(destination)
                .set(constant);
        }
        _ => unreachable!(),
    }

    if jump_next {
        *ip += 1;
    }
}

fn load_list(ip: &mut usize, instruction: &InstructionFields, thread: &mut Thread) {
    todo!()
}

fn load_function(_: &mut usize, _: &InstructionFields, _: &mut Thread) {
    todo!()
}

fn load_self(_: &mut usize, _: &InstructionFields, _: &mut Thread) {
    todo!()
}

fn subtract(_: &mut usize, instruction: &InstructionFields, thread: &mut Thread) {
    todo!()
}

fn multiply(_: &mut usize, instruction: &InstructionFields, thread: &mut Thread) {
    todo!()
}

fn divide(_: &mut usize, instruction: &InstructionFields, thread: &mut Thread) {
    todo!()
}

fn modulo(_: &mut usize, instruction: &InstructionFields, thread: &mut Thread) {
    todo!()
}

fn test(ip: &mut usize, instruction: &InstructionFields, thread: &mut Thread) {
    todo!()
}

fn test_set(_: &mut usize, _: &InstructionFields, _: &mut Thread) {
    todo!()
}

fn equal(ip: &mut usize, instruction: &InstructionFields, thread: &mut Thread) {
    todo!()
}

fn less_equal(ip: &mut usize, instruction: &InstructionFields, thread: &mut Thread) {
    todo!()
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
    let current_frame = thread.current_frame();

    if should_return_value {
        match return_type {
            TypeCode::BOOLEAN => {
                let return_value = current_frame
                    .get_boolean_from_register(return_register)
                    .clone_inner();
                thread.return_value = Some(Value::boolean(return_value));
            }
            TypeCode::BYTE => {
                let return_value = current_frame
                    .get_byte_from_register(return_register)
                    .clone_inner();
                thread.return_value = Some(Value::byte(return_value));
            }
            TypeCode::CHARACTER => {
                let return_value = current_frame
                    .get_character_from_register(return_register)
                    .clone_inner();
                thread.return_value = Some(Value::character(return_value));
            }
            TypeCode::FLOAT => {
                let return_value = current_frame
                    .get_float_from_register(return_register)
                    .clone_inner();
                thread.return_value = Some(Value::float(return_value));
            }
            TypeCode::INTEGER => {
                let return_value = current_frame
                    .get_integer_from_register(return_register)
                    .clone_inner();
                thread.return_value = Some(Value::integer(return_value));
            }
            TypeCode::STRING => {
                let return_value = current_frame
                    .get_string_from_register(return_register)
                    .clone_inner();
                thread.return_value = Some(Value::string(return_value));
            }
            TypeCode::LIST => {
                todo!()
            }
            _ => unreachable!(),
        }
    }
}
