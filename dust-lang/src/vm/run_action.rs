use tracing::trace;

use crate::{
    instruction::{
        Add, Call, CallNative, Close, Divide, Equal, GetLocal, Jump, Less, LessEqual, LoadBoolean,
        LoadConstant, LoadFunction, LoadList, LoadSelf, Modulo, Multiply, Negate, Not, Point,
        Return, SetLocal, Subtract, Test, TestSet,
    },
    vm::FunctionCall,
    AbstractList, Argument, ConcreteValue, Instruction, Type, Value,
};

use super::{thread::ThreadData, Pointer, Register};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RunAction {
    pub logic: RunnerLogic,
    pub instruction: Instruction,
}

impl From<Instruction> for RunAction {
    fn from(instruction: Instruction) -> Self {
        let operation = instruction.operation();
        let logic = RUNNER_LOGIC_TABLE[operation.0 as usize];

        RunAction { logic, instruction }
    }
}

pub type RunnerLogic = fn(Instruction, &mut ThreadData) -> bool;

pub const RUNNER_LOGIC_TABLE: [RunnerLogic; 25] = [
    point,
    close,
    load_boolean,
    load_constant,
    load_function,
    load_list,
    load_self,
    get_local,
    set_local,
    add,
    subtract,
    multiply,
    divide,
    modulo,
    test,
    test_set,
    equal,
    less,
    less_equal,
    negate,
    not,
    call,
    call_native,
    jump,
    r#return,
];

pub(crate) fn get_next_action(data: &mut ThreadData) -> RunAction {
    let current_call = data.call_stack.last_mut_unchecked();
    let instruction = current_call.chunk.instructions[current_call.ip];
    let operation = instruction.operation();
    let logic = RUNNER_LOGIC_TABLE[operation.0 as usize];

    current_call.ip += 1;

    RunAction { logic, instruction }
}

pub fn point(instruction: Instruction, data: &mut ThreadData) -> bool {
    let Point { from, to } = instruction.into();
    let from_register = data.get_register_unchecked(from);
    let from_register_is_empty = matches!(from_register, Register::Empty);

    if !from_register_is_empty {
        let register = Register::Pointer(Pointer::Register(to));

        data.set_register(from, register);
    }

    data.next_action = get_next_action(data);

    false
}

pub fn close(instruction: Instruction, data: &mut ThreadData) -> bool {
    let Close { from, to } = instruction.into();

    for register_index in from..to {
        data.set_register(register_index, Register::Empty);
    }

    data.next_action = get_next_action(data);

    false
}

pub fn load_boolean(instruction: Instruction, data: &mut ThreadData) -> bool {
    let LoadBoolean {
        destination,
        value,
        jump_next,
    } = instruction.into();
    let boolean = Value::Concrete(ConcreteValue::Boolean(value));
    let register = Register::Value(boolean);

    data.set_register(destination, register);

    if jump_next {
        let current_call = data.call_stack.last_mut_unchecked();

        current_call.ip += 1;
    }

    data.next_action = get_next_action(data);

    false
}

pub fn load_constant(instruction: Instruction, data: &mut ThreadData) -> bool {
    let LoadConstant {
        destination,
        constant_index,
        jump_next,
    } = instruction.into();
    let register = Register::Pointer(Pointer::Constant(constant_index));

    trace!("Load constant {constant_index} into R{destination}");

    data.set_register(destination, register);

    if jump_next {
        let current_call = data.call_stack.last_mut_unchecked();

        current_call.ip += 1;
    }

    data.next_action = get_next_action(data);

    false
}

pub fn load_list(instruction: Instruction, data: &mut ThreadData) -> bool {
    let LoadList {
        destination,
        start_register,
    } = instruction.into();
    let mut item_pointers = Vec::with_capacity((destination - start_register) as usize);
    let mut item_type = Type::Any;

    for register_index in start_register..destination {
        match data.get_register_unchecked(register_index) {
            Register::Empty => continue,
            Register::Value(value) => {
                if item_type == Type::Any {
                    item_type = value.r#type();
                }
            }
            Register::Pointer(pointer) => {
                if item_type == Type::Any {
                    item_type = data.follow_pointer_unchecked(*pointer).r#type();
                }
            }
        }

        let pointer = Pointer::Register(register_index);

        item_pointers.push(pointer);
    }

    let list_value = Value::AbstractList(AbstractList {
        item_type,
        item_pointers,
    });
    let register = Register::Value(list_value);

    data.set_register(destination, register);

    data.next_action = get_next_action(data);

    false
}

pub fn load_function(instruction: Instruction, data: &mut ThreadData) -> bool {
    let LoadFunction {
        destination,
        prototype_index,
    } = instruction.into();
    let prototype_index = prototype_index as usize;
    let current_call = data.call_stack.last_mut_unchecked();
    let prototype = &current_call.chunk.prototypes[prototype_index];
    let function = prototype.as_function();
    let register = Register::Value(Value::Function(function));

    data.set_register(destination, register);

    data.next_action = get_next_action(data);

    false
}

pub fn load_self(instruction: Instruction, data: &mut ThreadData) -> bool {
    let LoadSelf { destination } = instruction.into();
    let current_call = data.call_stack.last_mut_unchecked();
    let prototype = &current_call.chunk;
    let function = prototype.as_function();
    let register = Register::Value(Value::Function(function));

    data.set_register(destination, register);

    data.next_action = get_next_action(data);

    false
}

pub fn get_local(instruction: Instruction, data: &mut ThreadData) -> bool {
    let GetLocal {
        destination,
        local_index,
    } = instruction.into();
    let local_register_index = data.get_local_register(local_index);
    let register = Register::Pointer(Pointer::Register(local_register_index));

    data.set_register(destination, register);

    data.next_action = get_next_action(data);

    false
}

pub fn set_local(instruction: Instruction, data: &mut ThreadData) -> bool {
    let SetLocal {
        register_index,
        local_index,
    } = instruction.into();
    let local_register_index = data.get_local_register(local_index);
    let register = Register::Pointer(Pointer::Register(register_index));

    data.set_register(local_register_index, register);

    data.next_action = get_next_action(data);

    false
}

pub fn add(instruction: Instruction, data: &mut ThreadData) -> bool {
    let Add {
        destination,
        left,
        right,
    } = instruction.into();
    let left = data.get_argument_unchecked(left);
    let right = data.get_argument_unchecked(right);
    let sum = left.add(right);
    let register = Register::Value(sum);

    data.set_register(destination, register);

    data.next_action = get_next_action(data);

    false
}

pub fn subtract(instruction: Instruction, data: &mut ThreadData) -> bool {
    let Subtract {
        destination,
        left,
        right,
    } = instruction.into();
    let left = data.get_argument_unchecked(left);
    let right = data.get_argument_unchecked(right);
    let difference = left.subtract(right);
    let register = Register::Value(difference);

    data.set_register(destination, register);

    data.next_action = get_next_action(data);

    false
}

pub fn multiply(instruction: Instruction, data: &mut ThreadData) -> bool {
    let Multiply {
        destination,
        left,
        right,
    } = instruction.into();
    let left = data.get_argument_unchecked(left);
    let right = data.get_argument_unchecked(right);
    let product = match (left, right) {
        (Value::Concrete(left), Value::Concrete(right)) => match (left, right) {
            (ConcreteValue::Integer(left), ConcreteValue::Integer(right)) => {
                ConcreteValue::Integer(left * right).to_value()
            }
            _ => panic!("Value Error: Cannot multiply values"),
        },
        _ => panic!("Value Error: Cannot multiply values"),
    };
    let register = Register::Value(product);

    data.set_register(destination, register);

    data.next_action = get_next_action(data);

    false
}

pub fn divide(instruction: Instruction, data: &mut ThreadData) -> bool {
    let Divide {
        destination,
        left,
        right,
    } = instruction.into();
    let left = data.get_argument_unchecked(left);
    let right = data.get_argument_unchecked(right);
    let quotient = match (left, right) {
        (Value::Concrete(left), Value::Concrete(right)) => match (left, right) {
            (ConcreteValue::Integer(left), ConcreteValue::Integer(right)) => {
                ConcreteValue::Integer(left / right).to_value()
            }
            _ => panic!("Value Error: Cannot divide values"),
        },
        _ => panic!("Value Error: Cannot divide values"),
    };
    let register = Register::Value(quotient);

    data.set_register(destination, register);

    data.next_action = get_next_action(data);

    false
}

pub fn modulo(instruction: Instruction, data: &mut ThreadData) -> bool {
    let Modulo {
        destination,
        left,
        right,
    } = instruction.into();
    let left = data.get_argument_unchecked(left);
    let right = data.get_argument_unchecked(right);
    let remainder = match (left, right) {
        (Value::Concrete(left), Value::Concrete(right)) => match (left, right) {
            (ConcreteValue::Integer(left), ConcreteValue::Integer(right)) => {
                ConcreteValue::Integer(left % right).to_value()
            }
            _ => panic!("Value Error: Cannot modulo values"),
        },
        _ => panic!("Value Error: Cannot modulo values"),
    };
    let register = Register::Value(remainder);

    data.set_register(destination, register);

    data.next_action = get_next_action(data);

    false
}

pub fn test(instruction: Instruction, data: &mut ThreadData) -> bool {
    let Test {
        operand_register,
        test_value,
    } = instruction.into();
    let value = data.open_register_unchecked(operand_register);
    let boolean = if let Value::Concrete(ConcreteValue::Boolean(boolean)) = value {
        *boolean
    } else {
        panic!("VM Error: Expected boolean value for TEST operation",);
    };

    if boolean == test_value {
        let current_call = data.call_stack.last_mut_unchecked();

        current_call.ip += 1;
    }

    data.next_action = get_next_action(data);

    false
}

pub fn test_set(instruction: Instruction, data: &mut ThreadData) -> bool {
    let TestSet {
        destination,
        argument,
        test_value,
    } = instruction.into();
    let value = data.get_argument_unchecked(argument);
    let boolean = if let Value::Concrete(ConcreteValue::Boolean(boolean)) = value {
        *boolean
    } else {
        panic!("VM Error: Expected boolean value for TEST_SET operation",);
    };

    if boolean == test_value {
    } else {
        let pointer = match argument {
            Argument::Constant(constant_index) => Pointer::Constant(constant_index),
            Argument::Register(register_index) => Pointer::Register(register_index),
        };
        let register = Register::Pointer(pointer);

        data.set_register(destination, register);
    }

    data.next_action = get_next_action(data);

    false
}

pub fn equal(instruction: Instruction, data: &mut ThreadData) -> bool {
    let Equal { value, left, right } = instruction.into();
    let left = data.get_argument_unchecked(left);
    let right = data.get_argument_unchecked(right);
    let is_equal = left.equals(right);

    if is_equal == value {
        let current_call = data.call_stack.last_mut_unchecked();

        current_call.ip += 1;
    }

    data.next_action = get_next_action(data);

    false
}

pub fn less(instruction: Instruction, data: &mut ThreadData) -> bool {
    let Less { value, left, right } = instruction.into();
    let left = data.get_argument_unchecked(left);
    let right = data.get_argument_unchecked(right);
    let is_less = left < right;

    if is_less == value {
        let current_call = data.call_stack.last_mut_unchecked();

        current_call.ip += 1;
    }

    data.next_action = get_next_action(data);

    false
}

pub fn less_equal(instruction: Instruction, data: &mut ThreadData) -> bool {
    let LessEqual { value, left, right } = instruction.into();
    let left = data.get_argument_unchecked(left);
    let right = data.get_argument_unchecked(right);
    let is_less_or_equal = left <= right;

    if is_less_or_equal == value {
        let current_call = data.call_stack.last_mut_unchecked();

        current_call.ip += 1;
    }

    data.next_action = get_next_action(data);

    false
}

pub fn negate(instruction: Instruction, data: &mut ThreadData) -> bool {
    let Negate {
        destination,
        argument,
    } = instruction.into();
    let argument = data.get_argument_unchecked(argument);
    let negated = argument.negate();
    let register = Register::Value(negated);

    data.set_register(destination, register);

    data.next_action = get_next_action(data);

    false
}

pub fn not(instruction: Instruction, data: &mut ThreadData) -> bool {
    let Not {
        destination,
        argument,
    } = instruction.into();
    let argument = data.get_argument_unchecked(argument);
    let not = match argument {
        Value::Concrete(ConcreteValue::Boolean(boolean)) => ConcreteValue::Boolean(!boolean),
        _ => panic!("VM Error: Expected boolean value for NOT operation"),
    };
    let register = Register::Value(Value::Concrete(not));

    data.set_register(destination, register);

    data.next_action = get_next_action(data);

    false
}

pub fn jump(instruction: Instruction, data: &mut ThreadData) -> bool {
    let Jump {
        offset,
        is_positive,
    } = instruction.into();
    let offset = offset as usize;
    let current_call = data.call_stack.last_mut_unchecked();

    if is_positive {
        current_call.ip += offset;
    } else {
        current_call.ip -= offset + 1;
    }

    data.next_action = get_next_action(data);

    false
}

pub fn call(instruction: Instruction, data: &mut ThreadData) -> bool {
    let Call {
        destination: return_register,
        function_register,
        argument_count,
        is_recursive,
    } = instruction.into();
    let current_call = data.call_stack.last_unchecked();
    let first_argument_register = return_register - argument_count;
    let prototype = if is_recursive {
        current_call.chunk
    } else {
        let function = data
            .open_register_unchecked(function_register)
            .as_function()
            .unwrap();

        &current_call.chunk.prototypes[function.prototype_index as usize]
    };
    let mut next_call = FunctionCall::new(prototype, return_register);
    let mut argument_index = 0;

    for register_index in first_argument_register..return_register {
        let value_option = data.open_register_allow_empty_unchecked(register_index);
        let argument = if let Some(value) = value_option {
            value.clone()
        } else {
            continue;
        };

        next_call.registers[argument_index] = Register::Value(argument);
        argument_index += 1;
    }

    data.call_stack.push(next_call);

    data.next_action = get_next_action(data);

    false
}

pub fn call_native(instruction: Instruction, data: &mut ThreadData) -> bool {
    let CallNative {
        destination,
        function,
        argument_count,
    } = instruction.into();
    let first_argument_index = destination - argument_count;
    let argument_range = first_argument_index..destination;

    function.call(data, Some(destination), argument_range)
}

pub fn r#return(instruction: Instruction, data: &mut ThreadData) -> bool {
    trace!("Returning with call stack:\n{}", data.call_stack);

    let Return {
        should_return_value,
        return_register,
    } = instruction.into();
    let (destination, return_value) = if data.call_stack.len() == 1 {
        if should_return_value {
            data.return_value_index = Some(return_register);
        };

        return true;
    } else {
        let return_value = data.empty_register_or_clone_constant_unchecked(return_register);
        let destination = data.call_stack.pop_unchecked().return_register;

        (destination, return_value)
    };

    if should_return_value {
        data.set_register(destination, Register::Value(return_value));
    }

    data.next_action = get_next_action(data);

    false
}

#[cfg(test)]
mod tests {

    use crate::Operation;

    use super::*;

    const ALL_OPERATIONS: [(Operation, RunnerLogic); 24] = [
        (Operation::POINT, point),
        (Operation::CLOSE, close),
        (Operation::LOAD_BOOLEAN, load_boolean),
        (Operation::LOAD_CONSTANT, load_constant),
        (Operation::LOAD_LIST, load_list),
        (Operation::LOAD_SELF, load_self),
        (Operation::GET_LOCAL, get_local),
        (Operation::SET_LOCAL, set_local),
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
