use tracing::trace;

use crate::{
    AbstractList, Instruction, Operation, Type, Value,
    instruction::{InstructionBuilder, LoadBoolean, TypeCode},
};

use super::{Pointer, Register, thread::ThreadData};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Action {
    pub logic: ActionLogic,
    pub fields: InstructionBuilder,
}

impl From<&Instruction> for Action {
    fn from(instruction: &Instruction) -> Self {
        let logic = match instruction.operation() {
            Operation::POINT => point,
            Operation::CLOSE => close,
            Operation::LOAD_BOOLEAN => load_boolean,
            Operation::LOAD_CONSTANT => load_constant,
            Operation::LOAD_LIST => load_list,
            Operation::LOAD_FUNCTION => load_function,
            Operation::LOAD_SELF => load_self,
            Operation::GET_LOCAL => get_local,
            Operation::SET_LOCAL => set_local,
            Operation::ADD => add,
            Operation::JUMP => jump,
            Operation::RETURN => r#return,
            unknown => unknown.panic_from_unknown_code(),
        };
        let fields = InstructionBuilder {
            operation: instruction.operation(),
            b_is_constant: instruction.b_is_constant(),
            c_is_constant: instruction.c_is_constant(),
            d_field: instruction.d_field(),
            b_type: instruction.b_type(),
            c_type: instruction.c_type(),
            a_field: instruction.a_field(),
            b_field: instruction.b_field(),
            c_field: instruction.c_field(),
        };

        Action { logic, fields }
    }
}

pub type ActionLogic = fn(InstructionBuilder, &mut ThreadData);

pub fn point(fields: InstructionBuilder, data: &mut ThreadData) {
    todo!()
}

pub fn close(fields: InstructionBuilder, data: &mut ThreadData) {
    let from = fields.b_field;
    let to = fields.c_field;
    let r#type = fields.b_type;
    let current_frame = data.current_frame_mut();

    match r#type {
        TypeCode::INTEGER => {
            let registers = current_frame.registers.get_many_integer_mut(from, to);

            for register in registers {
                *register = Register::Empty;
            }
        }
        _ => todo!(),
    }
}

pub fn load_boolean(fields: InstructionBuilder, data: &mut ThreadData) {
    let destination = fields.a_field;
    let boolean = fields.b_field != 0;
    let new_register = Register::Value(boolean);
    let current_frame = data.current_frame_mut();
    let old_register = current_frame.registers.get_boolean_mut(destination);

    *old_register = new_register;

    if fields.c_field != 0 {
        current_frame.instruction_pointer += 1;
    }
}

pub fn load_constant(fields: InstructionBuilder, data: &mut ThreadData) {
    let destination = fields.a_field;
    let constant_index = fields.b_field;
    let r#type = fields.b_type;
    let current_frame = data.current_frame_mut();

    match r#type {
        TypeCode::INTEGER => {
            let value = current_frame
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

    if fields.c_field != 0 {
        current_frame.instruction_pointer += 1;
    }
}

pub fn load_list(fields: InstructionBuilder, data: &mut ThreadData) {
    todo!()
}

pub fn load_function(fields: InstructionBuilder, data: &mut ThreadData) {
    todo!()
}

pub fn load_self(fields: InstructionBuilder, data: &mut ThreadData) {
    todo!()
}

pub fn get_local(fields: InstructionBuilder, data: &mut ThreadData) {
    let destination = fields.a_field;
    let local_index = fields.b_field;
    let r#type = fields.b_type;
    let current_frame = data.current_frame_mut();

    match r#type {
        TypeCode::INTEGER => {
            let new_register = Register::Pointer::<i64>(Pointer::ConstantInteger(local_index));
            let old_register = current_frame.registers.get_integer_mut(destination);

            *old_register = new_register;
        }
        unknown => todo!(),
    }
}

pub fn set_local(fields: InstructionBuilder, data: &mut ThreadData) {
    let register_index = fields.b_field;
    let local_index = fields.c_field;
    let r#type = fields.b_type;
    let current_frame = data.current_frame_mut();

    match r#type {
        TypeCode::INTEGER => {
            let new_register = Register::Pointer::<i64>(Pointer::ConstantInteger(local_index));
            let old_register = current_frame.registers.get_integer_mut(register_index);

            *old_register = new_register;
        }
        unknown => todo!(),
    }
}

pub fn add(fields: InstructionBuilder, data: &mut ThreadData) {
    let destination = fields.a_field;
    let left_index = fields.b_field;
    let left_is_constant = fields.b_is_constant;
    let right_index = fields.c_field;
    let right_is_constant = fields.c_is_constant;
    let left_type = fields.b_type;
    let right_type = fields.c_type;
    let current_frame = data.current_frame_mut();

    match (left_type, right_type) {
        (TypeCode::INTEGER, TypeCode::INTEGER) => {
            let left = if left_is_constant {
                current_frame
                    .prototype
                    .constants
                    .get_integer(left_index)
                    .unwrap()
            } else {
                *current_frame
                    .registers
                    .get_integer(left_index)
                    .expect_value()
            };
            let right = if right_is_constant {
                current_frame
                    .prototype
                    .constants
                    .get_integer(right_index)
                    .unwrap()
            } else {
                *current_frame
                    .registers
                    .get_integer(right_index)
                    .expect_value()
            };
            let new_register = Register::Value(left + right);
            let old_register = current_frame.registers.get_integer_mut(destination);

            *old_register = new_register;
        }
        unknown => todo!(),
    }
}

pub fn jump(fields: InstructionBuilder, data: &mut ThreadData) {
    let offset = fields.b_field as usize;
    let is_positive = fields.c_field != 0;
    let current_frame = data.current_frame_mut();

    if is_positive {
        current_frame.instruction_pointer += offset;
    } else {
        current_frame.instruction_pointer -= offset + 1;
    }
}

pub fn r#return(fields: InstructionBuilder, data: &mut ThreadData) {
    trace!("Returning. Stack size = {}", data.stack.len());

    let should_return_value = fields.b_field != 0;
    let r#type = fields.b_type;
    let return_register = fields.c_field;
    let current_frame = data.stack.pop().unwrap();

    match r#type {
        TypeCode::INTEGER => {
            if data.stack.is_empty() {
                if should_return_value {
                    let return_value = current_frame
                        .registers
                        .get_integer(return_register)
                        .expect_value();

                    data.return_value = Some(Some(Value::Integer(*return_value)));
                } else {
                    data.return_value = Some(None);
                }

                return;
            }

            if should_return_value {
                let return_value = current_frame
                    .registers
                    .get_integer(return_register)
                    .expect_value();
                let outer_frame = data.current_frame_mut();
                let register = outer_frame.registers.get_integer_mut(return_register);

                *register = Register::Value(*return_value);
            }
        }
        _ => todo!(),
    }
}
