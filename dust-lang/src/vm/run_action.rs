use tracing::trace;

use crate::{
    instruction::{
        Add, Call, CallNative, Close, Divide, Equal, GetLocal, Jump, Less, LessEqual, LoadBoolean,
        LoadConstant, LoadFunction, LoadList, LoadSelf, Modulo, Multiply, Negate, Not, Point,
        Return, SetLocal, Subtract, Test, TestSet,
    },
    AbstractList, Argument, ConcreteValue, Instruction, Type, Value,
};

use super::{thread::ThreadSignal, Pointer, Record, Register};

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

pub type RunnerLogic = fn(Instruction, &mut Record) -> ThreadSignal;

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

pub fn point(instruction: Instruction, record: &mut Record) -> ThreadSignal {
    let Point { from, to } = instruction.into();
    let from_register = record.get_register(from);
    let from_register_is_empty = matches!(from_register, Register::Empty);

    if !from_register_is_empty {
        let register = Register::Pointer(Pointer::Stack(to));

        record.set_register(from, register);
    }

    ThreadSignal::Continue
}

pub fn close(instruction: Instruction, record: &mut Record) -> ThreadSignal {
    let Close { from, to } = instruction.into();

    assert!(from < to, "Runtime Error: Malformed instruction");

    for register_index in from..to {
        assert!(
            (register_index as usize) < record.stack_size(),
            "Runtime Error: Register index out of bounds"
        );

        record.set_register(register_index, Register::Empty);
    }

    ThreadSignal::Continue
}

pub fn load_boolean(instruction: Instruction, record: &mut Record) -> ThreadSignal {
    let LoadBoolean {
        destination,
        value,
        jump_next,
    } = instruction.into();
    let boolean = Value::Concrete(ConcreteValue::Boolean(value));
    let register = Register::Value(boolean);

    record.set_register(destination, register);

    if jump_next {
        record.ip += 1;
    }

    ThreadSignal::Continue
}

pub fn load_constant(instruction: Instruction, record: &mut Record) -> ThreadSignal {
    let LoadConstant {
        destination,
        constant_index,
        jump_next,
    } = instruction.into();
    let register = Register::Pointer(Pointer::Constant(constant_index));

    trace!("Load constant {constant_index} into R{destination}");

    record.set_register(destination, register);

    if jump_next {
        record.ip += 1;
    }

    ThreadSignal::Continue
}

pub fn load_list(instruction: Instruction, record: &mut Record) -> ThreadSignal {
    let LoadList {
        destination,
        start_register,
    } = instruction.into();
    let mut item_pointers = Vec::with_capacity((destination - start_register) as usize);
    let mut item_type = Type::Any;

    for register_index in start_register..destination {
        match record.get_register(register_index) {
            Register::Empty => continue,
            Register::Value(value) => {
                if item_type == Type::Any {
                    item_type = value.r#type();
                }
            }
            Register::Pointer(pointer) => {
                if item_type == Type::Any {
                    item_type = record.follow_pointer(*pointer).r#type();
                }
            }
        }

        let pointer = Pointer::Stack(register_index);

        item_pointers.push(pointer);
    }

    let list_value = Value::AbstractList(AbstractList {
        item_type,
        item_pointers,
    });
    let register = Register::Value(list_value);

    record.set_register(destination, register);

    ThreadSignal::Continue
}

pub fn load_function(instruction: Instruction, _: &mut Record) -> ThreadSignal {
    let LoadFunction {
        destination,
        prototype_index,
    } = instruction.into();

    ThreadSignal::LoadFunction {
        destination,
        prototype_index,
    }
}

pub fn load_self(instruction: Instruction, record: &mut Record) -> ThreadSignal {
    let LoadSelf { destination } = instruction.into();
    let function = record.as_function();
    let register = Register::Value(Value::Function(function));

    record.set_register(destination, register);

    ThreadSignal::Continue
}

pub fn get_local(instruction: Instruction, record: &mut Record) -> ThreadSignal {
    let GetLocal {
        destination,
        local_index,
    } = instruction.into();
    let local_register_index = record.get_local_register(local_index);
    let register = Register::Pointer(Pointer::Stack(local_register_index));

    record.set_register(destination, register);

    ThreadSignal::Continue
}

pub fn set_local(instruction: Instruction, record: &mut Record) -> ThreadSignal {
    let SetLocal {
        register_index,
        local_index,
    } = instruction.into();
    let local_register_index = record.get_local_register(local_index);
    let register = Register::Pointer(Pointer::Stack(register_index));

    record.set_register(local_register_index, register);

    ThreadSignal::Continue
}

pub fn add(instruction: Instruction, record: &mut Record) -> ThreadSignal {
    let Add {
        destination,
        left,
        right,
    } = instruction.into();
    let left = record.get_argument(left);
    let right = record.get_argument(right);
    let sum = left.add(right);
    let register = Register::Value(sum);

    record.set_register(destination, register);

    ThreadSignal::Continue
}

pub fn subtract(instruction: Instruction, record: &mut Record) -> ThreadSignal {
    let Subtract {
        destination,
        left,
        right,
    } = instruction.into();
    let left = record.get_argument(left);
    let right = record.get_argument(right);
    let difference = left.subtract(right);
    let register = Register::Value(difference);

    record.set_register(destination, register);

    ThreadSignal::Continue
}

pub fn multiply(instruction: Instruction, record: &mut Record) -> ThreadSignal {
    let Multiply {
        destination,
        left,
        right,
    } = instruction.into();
    let left = record.get_argument(left);
    let right = record.get_argument(right);
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

    record.set_register(destination, register);

    ThreadSignal::Continue
}

pub fn divide(instruction: Instruction, record: &mut Record) -> ThreadSignal {
    let Divide {
        destination,
        left,
        right,
    } = instruction.into();
    let left = record.get_argument(left);
    let right = record.get_argument(right);
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

    record.set_register(destination, register);

    ThreadSignal::Continue
}

pub fn modulo(instruction: Instruction, record: &mut Record) -> ThreadSignal {
    let Modulo {
        destination,
        left,
        right,
    } = instruction.into();
    let left = record.get_argument(left);
    let right = record.get_argument(right);
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

    record.set_register(destination, register);

    ThreadSignal::Continue
}

pub fn test(instruction: Instruction, record: &mut Record) -> ThreadSignal {
    let Test {
        operand_register,
        test_value,
    } = instruction.into();
    let value = record.open_register(operand_register);
    let boolean = if let Value::Concrete(ConcreteValue::Boolean(boolean)) = value {
        *boolean
    } else {
        panic!("VM Error: Expected boolean value for TEST operation",);
    };

    if boolean == test_value {
        record.ip += 1;
    }

    ThreadSignal::Continue
}

pub fn test_set(instruction: Instruction, record: &mut Record) -> ThreadSignal {
    let TestSet {
        destination,
        argument,
        test_value,
    } = instruction.into();
    let value = record.get_argument(argument);
    let boolean = if let Value::Concrete(ConcreteValue::Boolean(boolean)) = value {
        *boolean
    } else {
        panic!("VM Error: Expected boolean value for TEST_SET operation",);
    };

    if boolean == test_value {
        record.ip += 1;
    } else {
        let pointer = match argument {
            Argument::Constant(constant_index) => Pointer::Constant(constant_index),
            Argument::Register(register_index) => Pointer::Stack(register_index),
        };
        let register = Register::Pointer(pointer);

        record.set_register(destination, register);
    }

    ThreadSignal::Continue
}

pub fn equal(instruction: Instruction, record: &mut Record) -> ThreadSignal {
    let Equal { value, left, right } = instruction.into();
    let left = record.get_argument(left);
    let right = record.get_argument(right);
    let is_equal = left.equals(right);

    if is_equal == value {
        record.ip += 1;
    }

    ThreadSignal::Continue
}

pub fn less(instruction: Instruction, record: &mut Record) -> ThreadSignal {
    let Less { value, left, right } = instruction.into();
    let left = record.get_argument(left);
    let right = record.get_argument(right);
    let is_less = left < right;

    if is_less == value {
        record.ip += 1;
    }

    ThreadSignal::Continue
}

pub fn less_equal(instruction: Instruction, record: &mut Record) -> ThreadSignal {
    let LessEqual { value, left, right } = instruction.into();
    let left = record.get_argument(left);
    let right = record.get_argument(right);
    let is_less_or_equal = left <= right;

    if is_less_or_equal == value {
        record.ip += 1;
    }

    ThreadSignal::Continue
}

pub fn negate(instruction: Instruction, record: &mut Record) -> ThreadSignal {
    let Negate {
        destination,
        argument,
    } = instruction.into();
    let argument = record.get_argument(argument);
    let negated = argument.negate();
    let register = Register::Value(negated);

    record.set_register(destination, register);

    ThreadSignal::Continue
}

pub fn not(instruction: Instruction, record: &mut Record) -> ThreadSignal {
    let Not {
        destination,
        argument,
    } = instruction.into();
    let argument = record.get_argument(argument);
    let not = match argument {
        Value::Concrete(ConcreteValue::Boolean(boolean)) => ConcreteValue::Boolean(!boolean),
        _ => panic!("VM Error: Expected boolean value for NOT operation"),
    };
    let register = Register::Value(Value::Concrete(not));

    record.set_register(destination, register);

    ThreadSignal::Continue
}

pub fn jump(instruction: Instruction, record: &mut Record) -> ThreadSignal {
    let Jump {
        offset,
        is_positive,
    } = instruction.into();
    let offset = offset as usize;

    if is_positive {
        record.ip += offset;
    } else {
        record.ip -= offset + 1;
    }

    ThreadSignal::Continue
}

pub fn call(instruction: Instruction, _: &mut Record) -> ThreadSignal {
    let Call {
        destination: return_register,
        function_register,
        argument_count,
    } = instruction.into();

    ThreadSignal::Call {
        function_register,
        return_register,
        argument_count,
    }
}

pub fn call_native(instruction: Instruction, record: &mut Record) -> ThreadSignal {
    let CallNative {
        destination,
        function,
        argument_count,
    } = instruction.into();
    let first_argument_index = destination - argument_count;
    let argument_range = first_argument_index..destination;

    function
        .call(record, Some(destination), argument_range)
        .unwrap_or_else(|error| panic!("{error:?}"))
}

pub fn r#return(instruction: Instruction, _: &mut Record) -> ThreadSignal {
    let Return {
        should_return_value,
        return_register,
    } = instruction.into();

    ThreadSignal::Return {
        should_return_value,
        return_register,
    }
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
