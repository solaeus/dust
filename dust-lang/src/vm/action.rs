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

pub const RUNNER_LOGIC_TABLE: [RunnerLogic; 22] = [
    point,
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

pub fn point(instruction: InstructionBuilder, thread: &mut Thread) {
    let destination = instruction.a_field as usize;
    let to = instruction.b_field as usize;
    let to_is_constant = instruction.b_is_constant;
    let pointer = if to_is_constant {
        Pointer::Constant(to)
    } else {
        Pointer::Register(to)
    };
    let new_register = Register::Pointer(pointer);
    let old_register = thread.get_register_mut(destination);

    *old_register = new_register;
}

pub fn load_boolean(instruction: InstructionBuilder, thread: &mut Thread) {}

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

    const ALL_OPERATIONS: [(Operation, RunnerLogic); 21] = [
        (Operation::POINT, point),
        (Operation::LOAD_BOOLEAN, load_boolean),
        (Operation::LOAD_CONSTANT, load_constant),
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
