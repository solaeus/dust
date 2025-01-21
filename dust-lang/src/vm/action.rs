use std::{
    fmt::{self, Debug, Display, Formatter},
    ptr,
};

use smallvec::SmallVec;
use tracing::trace;

use crate::{
    Instruction, Operation, Type, Value,
    instruction::{InstructionBuilder, TypeCode},
};

use super::{Pointer, Register, thread::ThreadData};

#[derive(Debug)]
pub struct ActionSequence {
    pub actions: SmallVec<[Action; 128]>,
}

impl ActionSequence {
    pub fn new<T>(actions_iter: T) -> Self
    where
        T: Iterator<Item = Action> + ExactSizeIterator,
    {
        let (lower_bound, upper_bound) = actions_iter.size_hint();
        let mut actions = SmallVec::with_capacity(upper_bound.unwrap_or(lower_bound));

        for action in actions_iter {
            actions.push(action);
        }

        Self { actions }
    }

    pub fn get_mut(&mut self, index: usize) -> &mut Action {
        if cfg!(debug_assertions) {
            self.actions.get_mut(index).unwrap()
        } else {
            unsafe { self.actions.get_unchecked_mut(index) }
        }
    }
}

#[derive(Clone, Copy)]
pub struct Action {
    pub logic: ActionLogic,
    pub data: ActionData,
}

impl From<&Instruction> for Action {
    fn from(instruction: &Instruction) -> Self {
        let builder = InstructionBuilder::from(instruction);
        let operation = builder.operation;
        let logic = match operation {
            Operation::MOVE => point,
            Operation::CLOSE => close,
            Operation::LOAD_INLINE => load_inline,
            Operation::LOAD_CONSTANT => load_constant,
            Operation::LOAD_LIST => load_list,
            Operation::LOAD_FUNCTION => load_function,
            Operation::LOAD_SELF => load_self,
            Operation::GET_LOCAL => get_local,
            Operation::SET_LOCAL => set_local,
            Operation::ADD => add,
            Operation::LESS => less,
            Operation::JUMP => jump,
            Operation::RETURN => r#return,
            unknown => unknown.panic_from_unknown_code(),
        };

        Action {
            logic,
            data: ActionData {
                name: operation.name(),
                instruction: builder,
                pointers: [ptr::null_mut(); 3],
                runs: 0,
                next_optimized: None,
            },
        }
    }
}

impl Display for Action {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{}", self.data.name)
    }
}

impl Debug for Action {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{self}")
    }
}

#[derive(Debug, Clone, Copy)]
pub struct ActionData {
    pub name: &'static str,
    pub instruction: InstructionBuilder,

    pub pointers: [*mut i64; 3],
    pub runs: usize,

    pub next_optimized: Option<*mut Action>,
}

pub type ActionLogic = fn(&mut ThreadData, &mut ActionData);

fn point(thread_data: &mut ThreadData, action_data: &mut ActionData) {
    todo!()
}

#[allow(clippy::single_range_in_vec_init)]
fn close(thread_data: &mut ThreadData, action_data: &mut ActionData) {
    let ActionData { instruction, .. } = action_data;
    let current_frame = thread_data.current_frame_mut();
    let from = instruction.b_field as usize;
    let to = instruction.c_field as usize;
    let r#type = instruction.b_type;

    match r#type {
        TypeCode::INTEGER => {
            let [registers] = current_frame.registers.get_many_integer_mut([from..to]);

            for register in registers {
                *register = Register::Empty;
            }
        }
        _ => todo!(),
    }
}

fn load_inline(thread_data: &mut ThreadData, action_data: &mut ActionData) {
    let ActionData { instruction, .. } = action_data;
    let current_frame = thread_data.current_frame_mut();
    let destination = instruction.a_field;
    let boolean = instruction.b_field != 0;
    let new_register = Register::Value(boolean);
    let old_register = current_frame.registers.get_boolean_mut(destination);

    *old_register = new_register;

    if instruction.c_field != 0 {
        current_frame.instruction_pointer += 1;
    }
}

fn load_constant(thread_data: &mut ThreadData, action_data: &mut ActionData) {
    let ActionData { instruction, .. } = action_data;
    let current_frame = thread_data.current_frame_mut();
    let destination = instruction.a_field;
    let constant_index = instruction.b_field;
    let r#type = instruction.b_type;

    match r#type {
        TypeCode::INTEGER => {
            let value = *current_frame
                .prototype
                .constants
                .get_integer(constant_index)
                .unwrap();
            let new_register = Register::Value(value);
            let old_register = current_frame.registers.get_integer_mut(destination);

            *old_register = new_register;
        }
        unknown => todo!(),
    };

    if instruction.c_field != 0 {
        current_frame.instruction_pointer += 1;
    }
}

fn load_list(thread_data: &mut ThreadData, action_data: &mut ActionData) {
    todo!()
}

fn load_function(thread_data: &mut ThreadData, action_data: &mut ActionData) {
    todo!()
}

fn load_self(thread_data: &mut ThreadData, action_data: &mut ActionData) {
    todo!()
}

fn get_local(thread_data: &mut ThreadData, action_data: &mut ActionData) {
    let ActionData { instruction, .. } = action_data;
    let current_frame = thread_data.current_frame_mut();
    let destination = instruction.a_field;
    let local_index = instruction.b_field;
    let local = current_frame
        .prototype
        .locals
        .get(local_index as usize)
        .unwrap();

    match &local.r#type {
        Type::Boolean => {
            let new_register = Register::Pointer(Pointer::RegisterBoolean(local.register_index));
            let old_register = current_frame.registers.get_boolean_mut(destination);

            *old_register = new_register;
        }
        Type::Integer => {
            let new_register = Register::Pointer(Pointer::RegisterInteger(local.register_index));
            let old_register = current_frame.registers.get_integer_mut(destination);

            *old_register = new_register;
        }
        unknown => todo!(),
    }
}

fn set_local(thread_data: &mut ThreadData, action_data: &mut ActionData) {
    let ActionData { instruction, .. } = action_data;
    let current_frame = thread_data.current_frame_mut();
    let register_index = instruction.b_field;
    let local_index = instruction.c_field;
    let r#type = instruction.b_type;

    match r#type {
        TypeCode::INTEGER => {
            let new_register = Register::Pointer::<i64>(Pointer::ConstantInteger(local_index));
            let old_register = current_frame.registers.get_integer_mut(register_index);

            *old_register = new_register;
        }
        unknown => todo!(),
    }
}

fn add(thread_data: &mut ThreadData, action_data: &mut ActionData) {
    let ActionData {
        instruction,
        runs,
        pointers,
        ..
    } = action_data;

    if *runs > 0 {
        unsafe {
            *pointers[0] = *pointers[1] + *pointers[2];
        }

        return;
    }

    *runs += 1;

    let current_frame = thread_data.current_frame_mut();
    let destination = instruction.a_field;
    let left_is_constant = instruction.b_is_constant;
    let left_index = instruction.b_field as usize;
    let right_is_constant = instruction.c_is_constant;
    let right_index = instruction.c_field as usize;
    let (new_register, left_pointer, right_pointer) = match (left_is_constant, right_is_constant) {
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
            let new_register = Register::Value(left + right);

            (
                new_register,
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
            let new_register = Register::Value(left + *right);

            (
                new_register,
                Box::into_raw(Box::new(left)),
                right as *mut i64,
            )
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
            let new_register = Register::Value(*left + right);

            (
                new_register,
                left as *mut i64,
                Box::into_raw(Box::new(right)),
            )
        }
        (false, false) => {
            let [left, right] = current_frame
                .registers
                .get_many_integer_mut([left_index, right_index]);
            let left = left.expect_value_mut();
            let right = right.expect_value_mut();
            let new_register = Register::Value(*left + *right);

            (new_register, left as *mut i64, right as *mut i64)
        }
    };
    let old_register = current_frame.registers.get_integer_mut(destination);
    *old_register = new_register;
    pointers[0] = old_register.expect_value_mut();
    pointers[1] = left_pointer;
    pointers[2] = right_pointer;
}

fn less(thread_data: &mut ThreadData, action_data: &mut ActionData) {
    let ActionData {
        instruction,
        runs,
        pointers,
        ..
    } = action_data;
    let current_frame = thread_data.current_frame_mut();

    if *runs > 0 {
        unsafe {
            if *pointers[0] < *pointers[1] {
                current_frame.instruction_pointer += 1;
            }
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
                    let [left, right] = current_frame.registers.get_many_integer_mut([
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
            current_frame.instruction_pointer += 1;
        }

        pointers[0] = left_pointer;
        pointers[1] = right_pointer;
        *runs += 1;
    }
}

fn jump(thread_data: &mut ThreadData, action_data: &mut ActionData) {
    let ActionData { instruction, .. } = action_data;
    let current_frame = thread_data.current_frame_mut();
    let offset = instruction.b_field as usize;
    let is_positive = instruction.c_field != 0;

    if is_positive {
        current_frame.instruction_pointer += offset;
    } else {
        current_frame.instruction_pointer -= offset + 1;
    }
}

fn r#return(thread_data: &mut ThreadData, action_data: &mut ActionData) {
    trace!("Returning. Stack size = {}", thread_data.stack.len());

    let ActionData { instruction, .. } = action_data;
    let ThreadData {
        stack,
        return_value,
        spawned_threads,
    } = thread_data;
    let should_return_value = instruction.b_field != 0;
    let r#type = instruction.b_type;
    let return_register = instruction.c_field;
    let current_frame = stack.pop().unwrap();

    match r#type {
        TypeCode::BOOLEAN => {
            if stack.is_empty() {
                if should_return_value {
                    let boolean_return = current_frame
                        .registers
                        .get_boolean(return_register)
                        .expect_value();

                    *return_value = Some(Some(Value::Boolean(*boolean_return)));
                } else {
                    *return_value = Some(None);
                }

                return;
            }

            if should_return_value {
                let return_value = current_frame
                    .registers
                    .get_boolean(return_register)
                    .expect_value();
                let outer_frame = thread_data.current_frame_mut();
                let register = outer_frame.registers.get_boolean_mut(return_register);

                *register = Register::Value(*return_value);
            }
        }
        TypeCode::INTEGER => {
            if stack.is_empty() {
                if should_return_value {
                    let integer_return = current_frame
                        .registers
                        .get_integer(return_register)
                        .expect_value();

                    *return_value = Some(Some(Value::Integer(*integer_return)));
                } else {
                    *return_value = Some(None);
                }

                return;
            }

            if should_return_value {
                let return_value = current_frame
                    .registers
                    .get_integer(return_register)
                    .expect_value();
                let outer_frame = thread_data.current_frame_mut();
                let register = outer_frame.registers.get_integer_mut(return_register);

                *register = Register::Value(*return_value);
            }
        }
        _ => todo!(),
    }
}
