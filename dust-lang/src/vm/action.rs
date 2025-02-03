use tracing::trace;

use crate::{
    Instruction, Operand, Type, Value,
    instruction::{
        Add, Call, CallNative, Close, Divide, Equal, InstructionBuilder, Jump, Less, LessEqual,
        LoadConstant, LoadEncoded, LoadFunction, LoadList, LoadSelf, Modulo, Multiply, Negate, Not,
        Point, Return, Subtract, Test, TestSet, TypeCode,
    },
    vm::CallFrame,
};

use super::{Register, Thread, call_frame::Pointer};

pub struct ActionSequence {
    pub actions: Vec<Action>,
}

impl ActionSequence {
    pub fn new<'a, T: IntoIterator<Item = &'a Instruction>>(instructions: T) -> Self {
        let actions = instructions.into_iter().map(Action::from).collect();

        Self { actions }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Action {
    pub logic: ActionLogic,
    pub instruction: InstructionBuilder,
}

impl From<&Instruction> for Action {
    fn from(instruction: &Instruction) -> Self {
        let instruction = InstructionBuilder::from(instruction);
        let logic = RUNNER_LOGIC_TABLE[instruction.operation.0 as usize];

        Action { logic, instruction }
    }
}

pub type ActionLogic = fn(InstructionBuilder, &mut Thread);

pub const RUNNER_LOGIC_TABLE: [ActionLogic; 23] = [
    point,
    close,
    load_encoded,
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

pub fn point(instruction: InstructionBuilder, thread: &mut Thread) {
    let destination = instruction.a_field as usize;
    let to = instruction.b_field as usize;
    let r#type = instruction.b_type;

    match r#type {
        TypeCode::BOOLEAN => {
            thread.set_boolean_register(destination, Register::Pointer(Pointer::Register(to)));
        }
        TypeCode::BYTE => {
            thread.set_byte_register(destination, Register::Pointer(Pointer::Register(to)));
        }
        TypeCode::CHARACTER => {
            thread.set_character_register(destination, Register::Pointer(Pointer::Register(to)));
        }
        TypeCode::FLOAT => {
            thread.set_float_register(destination, Register::Pointer(Pointer::Register(to)));
        }
        TypeCode::INTEGER => {
            thread.set_integer_register(destination, Register::Pointer(Pointer::Register(to)));
        }
        TypeCode::STRING => {
            thread.set_string_register(destination, Register::Pointer(Pointer::Register(to)));
        }
        unsupported => panic!("Unsupported type code: {}", unsupported.0),
    }
}

pub fn close(instruction: InstructionBuilder, thread: &mut Thread) {}

pub fn load_encoded(instruction: InstructionBuilder, thread: &mut Thread) {}

pub fn load_constant(instruction: InstructionBuilder, thread: &mut Thread) {}

pub fn load_list(instruction: InstructionBuilder, thread: &mut Thread) {}

pub fn load_function(instruction: InstructionBuilder, thread: &mut Thread) {}

pub fn load_self(instruction: InstructionBuilder, thread: &mut Thread) {}

pub fn get_local(instruction: InstructionBuilder, thread: &mut Thread) {}

pub fn set_local(instruction: InstructionBuilder, thread: &mut Thread) {}

pub fn add(instruction: InstructionBuilder, thread: &mut Thread) {}

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

pub fn r#return(instruction: InstructionBuilder, thread: &mut Thread) {}

#[cfg(test)]
mod tests {

    use crate::Operation;

    use super::*;

    const ALL_OPERATIONS: [(Operation, ActionLogic); 23] = [
        (Operation::POINT, point),
        (Operation::CLOSE, close),
        (Operation::LOAD_ENCODED, load_encoded),
        (Operation::LOAD_CONSTANT, load_constant),
        (Operation::LOAD_LIST, load_list),
        (Operation::LOAD_FUNCTION, load_function),
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
