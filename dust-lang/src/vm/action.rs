use tracing::trace;

use crate::{
    AbstractList, ConcreteValue, Instruction, Operand, Type, Value,
    instruction::{
        Add, Call, CallNative, Divide, Equal, InstructionBuilder, Jump, Less, LessEqual,
        LoadBoolean, LoadConstant, LoadFunction, LoadList, LoadSelf, Modulo, Multiply, Negate, Not,
        Return, Subtract, Test, TestSet, TypeCode,
    },
    vm::CallFrame,
};

use super::{Pointer, Register, thread::Thread};

#[derive(Debug)]
pub struct ActionSequence {
    pub actions: Vec<Action>,
}

impl ActionSequence {
    pub fn new(instructions: &[Instruction]) -> Self {
        let mut actions = Vec::with_capacity(instructions.len());

        for instruction in instructions {
            let action = Action::from(instruction);

            actions.push(action);
        }

        ActionSequence { actions }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Action {
    pub logic: RunnerLogic,
    pub instruction: InstructionBuilder,
}

impl From<&Instruction> for Action {
    fn from(instruction: &Instruction) -> Self {
        let operation = instruction.operation();
        let logic = RUNNER_LOGIC_TABLE[operation.0 as usize];
        let instruction = InstructionBuilder::from(instruction);

        Action { logic, instruction }
    }
}

pub type RunnerLogic = fn(InstructionBuilder, &mut Thread);

pub const RUNNER_LOGIC_TABLE: [RunnerLogic; 23] = [
    point,
    close,
    load_boolean,
    load_constant,
    load_function,
    load_list,
    load_self,
    add,
    subtract,
    multiply,
    divide,
    modulo,
    equal,
    less,
    less_equal,
    negate,
    not,
    test,
    test_set,
    call,
    call_native,
    jump,
    r#return,
];

pub fn point(instruction: InstructionBuilder, thread: &mut Thread) {}

pub fn close(instruction: InstructionBuilder, thread: &mut Thread) {}

pub fn load_boolean(instruction: InstructionBuilder, thread: &mut Thread) {}

pub fn load_constant(instruction: InstructionBuilder, thread: &mut Thread) {}

pub fn load_list(instruction: InstructionBuilder, thread: &mut Thread) {}

pub fn load_function(instruction: InstructionBuilder, thread: &mut Thread) {}

pub fn load_self(instruction: InstructionBuilder, thread: &mut Thread) {}

pub fn add(instruction: InstructionBuilder, thread: &mut Thread) {
    let destination = instruction.a_field as usize;
    let left = instruction.b_field as usize;
    let left_is_constant = instruction.b_is_constant;
    let left_type = instruction.b_type;
    let right = instruction.c_field as usize;
    let right_is_constant = instruction.c_is_constant;
    let right_type = instruction.c_type;

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
            let result = left_value + right_value;

            println!("{left} + {right} = {destination}");
            println!("{left_value} + {right_value} = {result}");

            thread.set_integer_register(destination, result);
        }
        _ => unimplemented!(),
    }
}

pub fn subtract(instruction: InstructionBuilder, thread: &mut Thread) {}

pub fn multiply(instruction: InstructionBuilder, thread: &mut Thread) {}

pub fn divide(instruction: InstructionBuilder, thread: &mut Thread) {}

pub fn modulo(instruction: InstructionBuilder, thread: &mut Thread) {}

pub fn test(instruction: InstructionBuilder, thread: &mut Thread) {}

pub fn test_set(instruction: InstructionBuilder, thread: &mut Thread) {}

pub fn equal(instruction: InstructionBuilder, thread: &mut Thread) {}

pub fn less(instruction: InstructionBuilder, thread: &mut Thread) {}

pub fn less_equal(instruction: InstructionBuilder, thread: &mut Thread) {}

pub fn negate(instruction: InstructionBuilder, thread: &mut Thread) {}

pub fn not(instruction: InstructionBuilder, thread: &mut Thread) {}

pub fn jump(instruction: InstructionBuilder, thread: &mut Thread) {}

pub fn call(instruction: InstructionBuilder, thread: &mut Thread) {}

pub fn call_native(instruction: InstructionBuilder, thread: &mut Thread) {}

pub fn r#return(instruction: InstructionBuilder, thread: &mut Thread) {
    let should_return_value = instruction.b_field != 0;
    let return_register = instruction.c_field as usize;
    let return_type = instruction.b_type;

    if should_return_value {
        match return_type {
            TypeCode::BOOLEAN => {
                let return_value = thread.get_boolean_register(return_register);
                thread.return_value = Some(Some(Value::boolean(return_value)));
            }
            TypeCode::BYTE => {
                let return_value = thread.get_byte_register(return_register);
                thread.return_value = Some(Some(Value::byte(return_value)));
            }
            TypeCode::CHARACTER => {
                let return_value = thread.get_character_register(return_register);
                thread.return_value = Some(Some(Value::character(return_value)));
            }
            TypeCode::FLOAT => {
                let return_value = thread.get_float_register(return_register);
                thread.return_value = Some(Some(Value::float(return_value)));
            }
            TypeCode::INTEGER => {
                let return_value = thread.get_integer_register(return_register);
                thread.return_value = Some(Some(Value::integer(return_value)));
            }
            TypeCode::STRING => {
                let return_value = thread.get_string_register(return_register).clone();
                thread.return_value = Some(Some(Value::string(return_value)));
            }
            _ => unimplemented!(),
        }
    } else {
        thread.return_value = Some(None);
    }
}

#[cfg(test)]
mod tests {

    use crate::Operation;

    use super::*;

    const ALL_OPERATIONS: [(Operation, RunnerLogic); 23] = [
        (Operation::POINT, point),
        (Operation::CLOSE, close),
        (Operation::LOAD_BOOLEAN, load_boolean),
        (Operation::LOAD_CONSTANT, load_constant),
        (Operation::LOAD_FUNCTION, load_function),
        (Operation::LOAD_LIST, load_list),
        (Operation::LOAD_SELF, load_self),
        (Operation::ADD, add),
        (Operation::SUBTRACT, subtract),
        (Operation::MULTIPLY, multiply),
        (Operation::DIVIDE, divide),
        (Operation::MODULO, modulo),
        (Operation::TEST, test),
        (Operation::TEST_SET, test_set),
        (Operation::EQUAL, equal),
        (Operation::LESS, less),
        (Operation::LESS_EQUAL, less_equal),
        (Operation::NEGATE, negate),
        (Operation::NOT, not),
        (Operation::CALL, call),
        (Operation::CALL_NATIVE, call_native),
        (Operation::JUMP, jump),
        (Operation::RETURN, r#return),
    ];

    #[test]
    fn operations_map_to_the_correct_runner() {
        for (operation, expected_runner) in ALL_OPERATIONS {
            let actual_runner = RUNNER_LOGIC_TABLE[operation.0 as usize];

            assert_eq!(
                expected_runner, actual_runner,
                "{operation} runner is incorrect"
            );
        }
    }
}
