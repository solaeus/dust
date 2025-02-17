mod add;
mod equal;
mod jump;
mod less;
mod less_equal;

use add::{
    add_bytes, add_character_string, add_characters, add_floats, add_integers,
    add_string_character, add_strings, optimized_add_integer,
};
use equal::{
    equal_booleans, equal_bytes, equal_characters, equal_floats, equal_integers, equal_strings,
    optimized_equal_integers,
};
use jump::{jump, optimized_jump_backward, optimized_jump_forward};
use less::{
    less_booleans, less_bytes, less_characters, less_floats, less_integers, less_strings,
    optimized_less_integers,
};

use less_equal::{
    less_equal_booleans, less_equal_bytes, less_equal_characters, less_equal_floats,
    less_equal_integers, less_equal_strings, optimized_less_equal_integers,
};
use tracing::info;

use std::fmt::{self, Display, Formatter};

use crate::{instruction::TypeCode, AbstractList, ConcreteValue, Instruction, Operation, Value};

use super::{call_frame::RuntimeValue, thread::Thread, Pointer};

pub type ActionLogic = fn(&mut usize, &Instruction, &mut Thread, &mut Cache);

#[derive(Debug)]
pub struct ActionSequence {
    actions: Vec<Action>,
}

impl ActionSequence {
    pub fn new<T: IntoIterator<Item = Instruction> + DoubleEndedIterator<Item = Instruction>>(
        instructions: T,
    ) -> Self {
        let mut actions = Vec::new();
        let mut instructions_reversed = instructions.rev();

        while let Some(instruction) = instructions_reversed.next() {
            if instruction.operation() == Operation::JUMP {
                let backward_offset = instruction.b_field() as usize;
                let is_positive = instruction.c_field() != 0;

                if !is_positive {
                    let mut loop_actions = Vec::with_capacity(backward_offset);
                    let jump_action = Action::optimized(instruction);

                    loop_actions.push(jump_action);

                    for _ in 0..backward_offset {
                        let instruction = instructions_reversed.next().unwrap();
                        let action = Action::optimized(instruction);

                        loop_actions.push(action);
                    }

                    loop_actions.reverse();

                    let cache = Cache::LoopActions(ActionSequence {
                        actions: loop_actions,
                    });

                    let action = Action {
                        instruction,
                        logic: r#loop,
                        cache,
                    };

                    actions.push(action);

                    continue;
                }
            }

            let action = Action::unoptimized(instruction);

            actions.push(action);
        }

        actions.reverse();

        ActionSequence { actions }
    }

    pub fn run(&mut self, thread: &mut Thread) {
        let mut local_ip = 0;

        while local_ip < self.actions.len() {
            let action = if cfg!(debug_assertions) {
                self.actions.get_mut(local_ip).unwrap()
            } else {
                unsafe { self.actions.get_unchecked_mut(local_ip) }
            };
            local_ip += 1;

            info!("Run {action}");

            (action.logic)(
                &mut local_ip,
                &action.instruction,
                thread,
                &mut action.cache,
            );
        }
    }
}

impl Display for ActionSequence {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "[")?;

        for (index, action) in self.actions.iter().enumerate() {
            if index > 0 {
                write!(f, ", ")?;
            }

            write!(f, "{action}")?;
        }

        write!(f, "]")
    }
}

#[derive(Debug)]
pub struct Action {
    instruction: Instruction,
    logic: ActionLogic,
    cache: Cache,
}

#[derive(Debug)]
pub enum Cache {
    Empty,
    IntegerMath([RuntimeValue<i64>; 3]),
    IntegerComparison([RuntimeValue<i64>; 2]),
    LoopActions(ActionSequence),
}

impl Action {
    pub fn unoptimized(instruction: Instruction) -> Self {
        let logic = match instruction.operation() {
            Operation::POINT => point,
            Operation::CLOSE => close,
            Operation::LOAD_ENCODED => load_encoded,
            Operation::LOAD_CONSTANT => load_constant,
            Operation::LOAD_LIST => load_list,
            Operation::LOAD_FUNCTION => load_function,
            Operation::LOAD_SELF => load_self,
            Operation::ADD => match (instruction.b_type(), instruction.c_type()) {
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
            Operation::EQUAL => match (instruction.b_type(), instruction.c_type()) {
                (TypeCode::BOOLEAN, TypeCode::BOOLEAN) => equal_booleans,
                (TypeCode::BYTE, TypeCode::BYTE) => equal_bytes,
                (TypeCode::CHARACTER, TypeCode::CHARACTER) => equal_characters,
                (TypeCode::FLOAT, TypeCode::FLOAT) => equal_floats,
                (TypeCode::INTEGER, TypeCode::INTEGER) => equal_integers,
                (TypeCode::STRING, TypeCode::STRING) => equal_strings,
                _ => todo!(),
            },
            Operation::LESS => match (instruction.b_type(), instruction.c_type()) {
                (TypeCode::BOOLEAN, TypeCode::BOOLEAN) => less_booleans,
                (TypeCode::BYTE, TypeCode::BYTE) => less_bytes,
                (TypeCode::CHARACTER, TypeCode::CHARACTER) => less_characters,
                (TypeCode::FLOAT, TypeCode::FLOAT) => less_floats,
                (TypeCode::INTEGER, TypeCode::INTEGER) => less_integers,
                (TypeCode::STRING, TypeCode::STRING) => less_strings,
                _ => todo!(),
            },
            Operation::LESS_EQUAL => match (instruction.b_type(), instruction.c_type()) {
                (TypeCode::BOOLEAN, TypeCode::BOOLEAN) => less_equal_booleans,
                (TypeCode::BYTE, TypeCode::BYTE) => less_equal_bytes,
                (TypeCode::CHARACTER, TypeCode::CHARACTER) => less_equal_characters,
                (TypeCode::FLOAT, TypeCode::FLOAT) => less_equal_floats,
                (TypeCode::INTEGER, TypeCode::INTEGER) => less_equal_integers,
                (TypeCode::STRING, TypeCode::STRING) => less_equal_strings,
                _ => todo!(),
            },
            Operation::TEST => test,
            Operation::TEST_SET => test_set,
            Operation::CALL => call,
            Operation::CALL_NATIVE => call_native,
            Operation::JUMP => jump,
            Operation::RETURN => r#return,
            _ => todo!(),
        };

        Action {
            instruction,
            logic,
            cache: Cache::Empty,
        }
    }

    pub fn optimized(instruction: Instruction) -> Self {
        let logic = match instruction.operation() {
            Operation::JUMP => match instruction.c_field() {
                0 => optimized_jump_backward,
                _ => optimized_jump_forward,
            },
            Operation::ADD => match (instruction.b_type(), instruction.c_type()) {
                (TypeCode::INTEGER, TypeCode::INTEGER) => optimized_add_integer,
                _ => todo!(),
            },
            Operation::EQUAL => match (instruction.b_type(), instruction.c_type()) {
                (TypeCode::INTEGER, TypeCode::INTEGER) => optimized_equal_integers,
                _ => todo!(),
            },
            Operation::LESS => match (instruction.b_type(), instruction.c_type()) {
                (TypeCode::INTEGER, TypeCode::INTEGER) => optimized_less_integers,
                _ => todo!(),
            },
            Operation::LESS_EQUAL => match (instruction.b_type(), instruction.c_type()) {
                (TypeCode::INTEGER, TypeCode::INTEGER) => optimized_less_equal_integers,
                _ => todo!(),
            },
            _ => todo!(),
        };

        Action {
            instruction,
            logic,
            cache: Cache::Empty,
        }
    }
}

impl Display for Action {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        if let Cache::LoopActions(actions) = &self.cache {
            write!(f, "LOOP: {actions}")?;
        } else {
            write!(f, "{}", self.instruction.operation())?;
        }

        Ok(())
    }
}

fn r#loop(_: &mut usize, _: &Instruction, thread: &mut Thread, cache: &mut Cache) {
    if let Cache::LoopActions(actions) = cache {
        actions.run(thread);
    }
}

fn point(_: &mut usize, instruction: &Instruction, thread: &mut Thread, _: &mut Cache) {
    todo!()
}

fn close(_: &mut usize, instruction: &Instruction, thread: &mut Thread, _: &mut Cache) {
    todo!()
}

fn load_encoded(ip: &mut usize, instruction: &Instruction, thread: &mut Thread, _: &mut Cache) {
    let destination = instruction.a_field();
    let value_type = instruction.b_type();
    let jump_next = instruction.c_field() != 0;

    match value_type {
        TypeCode::BOOLEAN => {
            let value = instruction.b_field() != 0;

            thread
                .current_frame_mut()
                .registers
                .booleans
                .get_mut(destination as usize)
                .as_value_mut()
                .set_inner(value);
        }
        TypeCode::BYTE => {
            let value = instruction.b_field() as u8;

            thread
                .current_frame_mut()
                .registers
                .bytes
                .get_mut(destination as usize)
                .as_value_mut()
                .set_inner(value);
        }
        _ => unreachable!(),
    }

    if jump_next {
        *ip += 1;
    }
}

fn load_constant(ip: &mut usize, instruction: &Instruction, thread: &mut Thread, _: &mut Cache) {
    let destination = instruction.a_field() as usize;
    let constant_index = instruction.b_field() as usize;
    let constant_type = instruction.b_type();
    let jump_next = instruction.c_field() != 0;
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

fn load_list(ip: &mut usize, instruction: &Instruction, thread: &mut Thread, _: &mut Cache) {
    let destination = instruction.a_field() as usize;
    let start_register = instruction.b_field() as usize;
    let item_type = instruction.b_type();
    let end_register = instruction.c_field() as usize;
    let jump_next = instruction.d_field();
    let current_frame = thread.current_frame_mut();

    let mut item_pointers = Vec::with_capacity(end_register - start_register + 1);

    match item_type {
        TypeCode::BOOLEAN => {
            for register_index in start_register..=end_register {
                let register_is_closed = current_frame.registers.booleans.is_closed(register_index);

                if register_is_closed {
                    continue;
                }

                item_pointers.push(Pointer::Register(register_index));
            }
        }
        TypeCode::BYTE => {
            for register_index in start_register..=end_register {
                let register_is_closed = current_frame.registers.bytes.is_closed(register_index);

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
                let register_is_closed = current_frame.registers.floats.is_closed(register_index);

                if register_is_closed {
                    continue;
                }

                item_pointers.push(Pointer::Register(register_index));
            }
        }
        TypeCode::INTEGER => {
            for register_index in start_register..=end_register {
                let register_is_closed = current_frame.registers.integers.is_closed(register_index);

                if register_is_closed {
                    continue;
                }

                item_pointers.push(Pointer::Register(register_index));
            }
        }
        TypeCode::STRING => {
            for register_index in start_register..=end_register {
                let register_is_closed = current_frame.registers.strings.is_closed(register_index);

                if register_is_closed {
                    continue;
                }

                item_pointers.push(Pointer::Register(register_index));
            }
        }
        _ => unreachable!(),
    }

    let list = RuntimeValue::Raw(AbstractList {
        item_type,
        item_pointers,
    });

    current_frame.registers.lists.get_mut(destination).set(list);

    if jump_next {
        *ip += 1;
    }
}

fn load_function(_: &mut usize, _: &Instruction, _: &mut Thread, _: &mut Cache) {
    todo!()
}

fn load_self(_: &mut usize, _: &Instruction, _: &mut Thread, _: &mut Cache) {
    todo!()
}

fn subtract(_: &mut usize, instruction: &Instruction, thread: &mut Thread, _: &mut Cache) {
    todo!()
}

fn multiply(_: &mut usize, instruction: &Instruction, thread: &mut Thread, _: &mut Cache) {
    todo!()
}

fn divide(_: &mut usize, instruction: &Instruction, thread: &mut Thread, _: &mut Cache) {
    todo!()
}

fn modulo(_: &mut usize, instruction: &Instruction, thread: &mut Thread, _: &mut Cache) {
    todo!()
}

fn test(ip: &mut usize, instruction: &Instruction, thread: &mut Thread, _: &mut Cache) {
    todo!()
}

fn test_set(_: &mut usize, _: &Instruction, _: &mut Thread, _: &mut Cache) {
    todo!()
}

fn negate(_: &mut usize, _: &Instruction, _: &mut Thread, _: &mut Cache) {
    todo!()
}

fn not(_: &mut usize, _: &Instruction, _: &mut Thread, _: &mut Cache) {
    todo!()
}

fn call(_: &mut usize, _: &Instruction, _: &mut Thread, _: &mut Cache) {
    todo!()
}

fn call_native(_: &mut usize, _: &Instruction, _: &mut Thread, _: &mut Cache) {
    todo!()
}

fn r#return(_: &mut usize, instruction: &Instruction, thread: &mut Thread, _: &mut Cache) {
    let should_return_value = instruction.b_field() != 0;
    let return_register = instruction.c_field() as usize;
    let return_type = instruction.b_type();
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
                let abstract_list = current_frame
                    .get_list_from_register(return_register)
                    .clone_inner();
                let mut concrete_list = Vec::with_capacity(abstract_list.item_pointers.len());

                match abstract_list.item_type {
                    TypeCode::BOOLEAN => {
                        for pointer in abstract_list.item_pointers {
                            let boolean = current_frame
                                .get_boolean_from_pointer(&pointer)
                                .clone_inner();
                            let value = ConcreteValue::Boolean(boolean);

                            concrete_list.push(value);
                        }
                    }
                    TypeCode::BYTE => {
                        for pointer in abstract_list.item_pointers {
                            let byte = current_frame.get_byte_from_pointer(&pointer).clone_inner();
                            let value = ConcreteValue::Byte(byte);

                            concrete_list.push(value);
                        }
                    }
                    TypeCode::CHARACTER => {
                        for pointer in abstract_list.item_pointers {
                            let character = current_frame
                                .get_character_from_pointer(&pointer)
                                .clone_inner();
                            let value = ConcreteValue::Character(character);

                            concrete_list.push(value);
                        }
                    }
                    TypeCode::FLOAT => {
                        for pointer in abstract_list.item_pointers {
                            let float =
                                current_frame.get_float_from_pointer(&pointer).clone_inner();
                            let value = ConcreteValue::Float(float);

                            concrete_list.push(value);
                        }
                    }
                    TypeCode::INTEGER => {
                        for pointer in abstract_list.item_pointers {
                            let integer = current_frame
                                .get_integer_from_pointer(&pointer)
                                .clone_inner();
                            let value = ConcreteValue::Integer(integer);

                            concrete_list.push(value);
                        }
                    }
                    TypeCode::STRING => {
                        for pointer in abstract_list.item_pointers {
                            let string = current_frame
                                .get_string_from_pointer(&pointer)
                                .clone_inner();
                            let value = ConcreteValue::String(string);

                            concrete_list.push(value);
                        }
                    }
                    _ => todo!(),
                }

                thread.return_value = Some(Value::Concrete(ConcreteValue::list(
                    concrete_list,
                    abstract_list.item_type,
                )));
            }
            _ => unreachable!(),
        }
    }
}
