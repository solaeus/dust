use smallvec::SmallVec;

use crate::{
    instruction::{
        Call, CallNative, Close, LoadBoolean, LoadConstant, LoadFunction, LoadList, LoadSelf, Point,
    },
    vm::VmError,
    AbstractList, ConcreteValue, Function, Instruction, InstructionData, NativeFunction, Type,
    Value,
};

use super::{thread::ThreadSignal, FunctionCall, Pointer, Record, Register};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RunAction {
    pub logic: RunnerLogic,
    pub data: InstructionData,
}

impl From<&Instruction> for RunAction {
    fn from(instruction: &Instruction) -> Self {
        let (operation, data) = instruction.decode();
        let logic = RUNNER_LOGIC_TABLE[operation.0 as usize];

        RunAction { logic, data }
    }
}

impl From<Instruction> for RunAction {
    fn from(instruction: Instruction) -> Self {
        let (operation, data) = instruction.decode();
        let logic = RUNNER_LOGIC_TABLE[operation.0 as usize];

        RunAction { logic, data }
    }
}

pub type RunnerLogic = fn(InstructionData, &mut Record) -> ThreadSignal;

pub const RUNNER_LOGIC_TABLE: [RunnerLogic; 25] = [
    r#move,
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

pub fn r#move(instruction_data: InstructionData, record: &mut Record) -> ThreadSignal {
    let Point { from, to } = instruction_data.into();
    let from_register = record.get_register(from);
    let from_register_is_empty = matches!(from_register, Register::Empty);

    if !from_register_is_empty {
        let register = Register::Pointer(Pointer::Stack(to));

        record.set_register(from, register);
    }

    ThreadSignal::Continue
}

pub fn close(instruction_data: InstructionData, record: &mut Record) -> ThreadSignal {
    let Close { from, to } = instruction_data.into();

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

pub fn load_boolean(instruction_data: InstructionData, record: &mut Record) -> ThreadSignal {
    let LoadBoolean {
        destination,
        value,
        jump_next,
    } = instruction_data.into();
    let boolean = Value::Concrete(ConcreteValue::Boolean(value));
    let register = Register::Value(boolean);

    record.set_register(destination, register);

    if jump_next {
        record.ip += 1;
    }

    ThreadSignal::Continue
}

pub fn load_constant(instruction_data: InstructionData, record: &mut Record) -> ThreadSignal {
    let LoadConstant {
        destination,
        constant_index,
        jump_next,
    } = instruction_data.into();
    let register = Register::Pointer(Pointer::Constant(constant_index));

    record.set_register(destination, register);

    if jump_next {
        record.ip += 1;
    }

    ThreadSignal::Continue
}

pub fn load_list(instruction_data: InstructionData, record: &mut Record) -> ThreadSignal {
    let LoadList {
        destination,
        start_register,
    } = instruction_data.into();
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

pub fn load_function(instruction_data: InstructionData, record: &mut Record) -> ThreadSignal {
    let LoadFunction {
        destination,
        record_index,
    } = instruction_data.into();

    ThreadSignal::Continue
}

pub fn load_self(instruction_data: InstructionData, record: &mut Record) -> ThreadSignal {
    let LoadSelf { destination } = instruction_data.into();
    let register = Register::Value(Value::SelfFunction);

    record.set_register(destination, register);

    ThreadSignal::Continue
}

pub fn get_local(instruction_data: InstructionData, record: &mut Record) -> ThreadSignal {
    let InstructionData {
        a_field: a,
        b_field: b,
        ..
    } = instruction_data;
    let local_register_index = record.get_local_register(b);
    let register = Register::Pointer(Pointer::Stack(local_register_index));

    record.set_register(a, register);

    ThreadSignal::Continue
}

pub fn set_local(instruction_data: InstructionData, record: &mut Record) -> ThreadSignal {
    let InstructionData {
        b_field: b,
        c_field: c,
        ..
    } = instruction_data;
    let local_register_index = record.get_local_register(c);
    let register = Register::Pointer(Pointer::Stack(b));

    record.set_register(local_register_index, register);

    ThreadSignal::Continue
}

pub fn add(instruction_data: InstructionData, record: &mut Record) -> ThreadSignal {
    let InstructionData {
        a_field: a,
        b_field: b,
        c_field: c,
        b_is_constant,
        c_is_constant,
        ..
    } = instruction_data;
    let left = record.get_argument(b, b_is_constant);
    let right = record.get_argument(c, c_is_constant);
    let sum = match (left, right) {
        (Value::Concrete(left), Value::Concrete(right)) => match (left, right) {
            (ConcreteValue::Integer(left), ConcreteValue::Integer(right)) => {
                ConcreteValue::Integer(left + right).to_value()
            }
            _ => panic!("Value Error: Cannot add values"),
        },
        _ => panic!("Value Error: Cannot add values {left} and {right}"),
    };
    let register = Register::Value(sum);

    record.set_register(a, register);

    ThreadSignal::Continue
}

pub fn subtract(instruction_data: InstructionData, record: &mut Record) -> ThreadSignal {
    let InstructionData {
        a_field: a,
        b_field: b,
        c_field: c,
        b_is_constant,
        c_is_constant,
        ..
    } = instruction_data;
    let left = record.get_argument(b, b_is_constant);
    let right = record.get_argument(c, c_is_constant);
    let difference = match (left, right) {
        (Value::Concrete(left), Value::Concrete(right)) => match (left, right) {
            (ConcreteValue::Integer(left), ConcreteValue::Integer(right)) => {
                ConcreteValue::Integer(left - right).to_value()
            }
            _ => panic!("Value Error: Cannot subtract values {left} and {right}"),
        },
        _ => panic!("Value Error: Cannot subtract values {left} and {right}"),
    };
    let register = Register::Value(difference);

    record.set_register(a, register);

    ThreadSignal::Continue
}

pub fn multiply(instruction_data: InstructionData, record: &mut Record) -> ThreadSignal {
    let InstructionData {
        a_field: a,
        b_field: b,
        c_field: c,
        b_is_constant,
        c_is_constant,
        ..
    } = instruction_data;
    let left = record.get_argument(b, b_is_constant);
    let right = record.get_argument(c, c_is_constant);
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

    record.set_register(a, register);

    ThreadSignal::Continue
}

pub fn divide(instruction_data: InstructionData, record: &mut Record) -> ThreadSignal {
    let InstructionData {
        a_field: a,
        b_field: b,
        c_field: c,
        b_is_constant,
        c_is_constant,
        ..
    } = instruction_data;
    let left = record.get_argument(b, b_is_constant);
    let right = record.get_argument(c, c_is_constant);
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

    record.set_register(a, register);

    ThreadSignal::Continue
}

pub fn modulo(instruction_data: InstructionData, record: &mut Record) -> ThreadSignal {
    let InstructionData {
        a_field: a,
        b_field: b,
        c_field: c,
        b_is_constant,
        c_is_constant,
        ..
    } = instruction_data;
    let left = record.get_argument(b, b_is_constant);
    let right = record.get_argument(c, c_is_constant);
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

    record.set_register(a, register);

    ThreadSignal::Continue
}

pub fn test(instruction_data: InstructionData, record: &mut Record) -> ThreadSignal {
    let InstructionData {
        b_field: b,
        b_is_constant,
        c_field: c,
        ..
    } = instruction_data;
    let value = record.get_argument(b, b_is_constant);
    let boolean = if let Value::Concrete(ConcreteValue::Boolean(boolean)) = value {
        *boolean
    } else {
        panic!("VM Error: Expected boolean value for TEST operation",);
    };
    let test_value = c != 0;

    if boolean == test_value {
        record.ip += 1;
    }

    ThreadSignal::Continue
}

pub fn test_set(instruction_data: InstructionData, record: &mut Record) -> ThreadSignal {
    let InstructionData {
        a_field: a,
        b_field: b,
        c_field: c,
        b_is_constant,
        ..
    } = instruction_data;
    let value = record.get_argument(b, b_is_constant);
    let boolean = if let Value::Concrete(ConcreteValue::Boolean(boolean)) = value {
        *boolean
    } else {
        panic!("VM Error: Expected boolean value for TEST_SET operation",);
    };
    let test_value = c != 0;

    if boolean == test_value {
        record.ip += 1;
    } else {
        let pointer = if b_is_constant {
            Pointer::Constant(b)
        } else {
            Pointer::Stack(b)
        };
        let register = Register::Pointer(pointer);

        record.set_register(a, register);
    }

    ThreadSignal::Continue
}

pub fn equal(instruction_data: InstructionData, record: &mut Record) -> ThreadSignal {
    let InstructionData {
        b_field: b,
        c_field: c,
        b_is_constant,
        c_is_constant,
        d_field: d,
        ..
    } = instruction_data;
    let left = record.get_argument(b, b_is_constant);
    let right = record.get_argument(c, c_is_constant);
    let is_equal = left == right;

    if is_equal == d {
        record.ip += 1;
    }

    ThreadSignal::Continue
}

pub fn less(instruction_data: InstructionData, record: &mut Record) -> ThreadSignal {
    let InstructionData {
        b_field: b,
        c_field: c,
        b_is_constant,
        c_is_constant,
        d_field: d,
        ..
    } = instruction_data;
    let left = record.get_argument(b, b_is_constant);
    let right = record.get_argument(c, c_is_constant);
    let is_less = left < right;

    if is_less == d {
        record.ip += 1;
    }

    ThreadSignal::Continue
}

pub fn less_equal(instruction_data: InstructionData, record: &mut Record) -> ThreadSignal {
    let InstructionData {
        b_field: b,
        c_field: c,
        b_is_constant,
        c_is_constant,
        d_field: d,
        ..
    } = instruction_data;
    let left = record.get_argument(b, b_is_constant);
    let right = record.get_argument(c, c_is_constant);
    let is_less_or_equal = left <= right;

    if is_less_or_equal == d {
        record.ip += 1;
    }

    ThreadSignal::Continue
}

pub fn negate(instruction_data: InstructionData, record: &mut Record) -> ThreadSignal {
    let InstructionData {
        a_field: a,
        b_field: b,
        b_is_constant,
        ..
    } = instruction_data;
    let argument = record.get_argument(b, b_is_constant);
    let negated = argument.negate();
    let register = Register::Value(negated);

    record.set_register(a, register);

    ThreadSignal::Continue
}

pub fn not(instruction_data: InstructionData, record: &mut Record) -> ThreadSignal {
    let InstructionData {
        a_field: a,
        b_field: b,
        b_is_constant,
        ..
    } = instruction_data;
    let argument = record.get_argument(b, b_is_constant);
    let not = match argument {
        Value::Concrete(ConcreteValue::Boolean(boolean)) => ConcreteValue::Boolean(!boolean),
        _ => panic!("VM Error: Expected boolean value for NOT operation"),
    };
    let register = Register::Value(Value::Concrete(not));

    record.set_register(a, register);

    ThreadSignal::Continue
}

pub fn jump(instruction_data: InstructionData, record: &mut Record) -> ThreadSignal {
    let InstructionData {
        b_field: b,
        c_field: c,
        ..
    } = instruction_data;
    let offset = b as usize;
    let is_positive = c != 0;

    if is_positive {
        record.ip += offset + 1
    } else {
        record.ip -= offset
    }

    ThreadSignal::Continue
}

pub fn call(instruction_data: InstructionData, record: &mut Record) -> ThreadSignal {
    let Call {
        destination,
        function_register,
        argument_count,
    } = instruction_data.into();
    let function_value = record.open_register(function_register);
    let function = match function_value {
        Value::Function(function) => function,
        _ => panic!(
            "{}",
            VmError::ExpectedFunction {
                value: function_value.clone()
            }
        ),
    };

    ThreadSignal::Call(FunctionCall {
        record_index: function.record_index,
        return_register: destination,
        argument_count,
    })
}

pub fn call_native(instruction_data: InstructionData, record: &mut Record) -> ThreadSignal {
    let CallNative {
        destination,
        function,
        argument_count,
    } = instruction_data.into();
    let first_argument_index = destination - argument_count;
    let argument_range = first_argument_index..destination;

    let function = NativeFunction::from(function);
    let thread_signal = function
        .call(record, Some(destination), argument_range)
        .unwrap_or_else(|error| panic!("{error:?}"));

    thread_signal
}

pub fn r#return(instruction_data: InstructionData, _: &mut Record) -> ThreadSignal {
    let should_return_value = instruction_data.b_field != 0;

    ThreadSignal::Return(should_return_value)
}

#[cfg(test)]
mod tests {

    use crate::Operation;

    use super::*;

    const ALL_OPERATIONS: [(Operation, RunnerLogic); 24] = [
        (Operation::POINT, r#move),
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
