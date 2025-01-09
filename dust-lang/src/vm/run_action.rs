use tracing::trace;

use crate::{
    instruction::{
        Add, Call, CallNative, Close, Divide, Equal, GetLocal, Jump, Less, LessEqual, LoadBoolean,
        LoadConstant, LoadFunction, LoadList, LoadSelf, Modulo, Multiply, Negate, Not, Point,
        Return, SetLocal, Subtract, Test, TestSet,
    },
    vm::FunctionCall,
    AbstractList, Argument, ConcreteValue, DustString, Instruction, Type, Value,
};

use super::{thread::ThreadData, Pointer, Record, Register};

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

pub(crate) fn get_next_action(record: &mut Record) -> RunAction {
    let instruction = record.chunk.instructions[record.ip];
    let operation = instruction.operation();
    let logic = RUNNER_LOGIC_TABLE[operation.0 as usize];

    record.ip += 1;

    RunAction { logic, instruction }
}

pub fn point(instruction: Instruction, data: &mut ThreadData) -> bool {
    let record = &mut data.call_stack.last_mut_unchecked().record;
    let Point { from, to } = instruction.into();
    let from_register = record.get_register_unchecked(from);
    let from_register_is_empty = matches!(from_register, Register::Empty);

    if !from_register_is_empty {
        let register = Register::Pointer(Pointer::Stack(to));

        record.set_register(from, register);
    }

    data.next_action = get_next_action(record);

    false
}

pub fn close(instruction: Instruction, data: &mut ThreadData) -> bool {
    let record = &mut data.call_stack.last_mut_unchecked().record;
    let Close { from, to } = instruction.into();

    for register_index in from..to {
        record.set_register(register_index, Register::Empty);
    }

    data.next_action = get_next_action(record);

    false
}

pub fn load_boolean(instruction: Instruction, data: &mut ThreadData) -> bool {
    let record = &mut data.call_stack.last_mut_unchecked().record;
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

    data.next_action = get_next_action(record);

    false
}

pub fn load_constant(instruction: Instruction, data: &mut ThreadData) -> bool {
    let record = &mut data.call_stack.last_mut_unchecked().record;
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

    data.next_action = get_next_action(record);

    false
}

pub fn load_list(instruction: Instruction, data: &mut ThreadData) -> bool {
    let record = &mut data.call_stack.last_mut_unchecked().record;
    let LoadList {
        destination,
        start_register,
    } = instruction.into();
    let mut item_pointers = Vec::with_capacity((destination - start_register) as usize);
    let mut item_type = Type::Any;

    for register_index in start_register..destination {
        match record.get_register_unchecked(register_index) {
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

    data.next_action = get_next_action(record);

    false
}

pub fn load_function(instruction: Instruction, data: &mut ThreadData) -> bool {
    let record = &mut data.call_stack.last_mut_unchecked().record;
    let LoadFunction {
        destination,
        prototype_index,
    } = instruction.into();
    let prototype_index = prototype_index as usize;
    let function = record.chunk.prototypes[prototype_index].as_function();
    let register = Register::Value(Value::Function(function));

    record.set_register(destination, register);

    data.next_action = get_next_action(record);

    false
}

pub fn load_self(instruction: Instruction, data: &mut ThreadData) -> bool {
    let record = &mut data.call_stack.last_mut_unchecked().record;
    let LoadSelf { destination } = instruction.into();
    let function = record.as_function();
    let register = Register::Value(Value::Function(function));

    record.set_register(destination, register);

    data.next_action = get_next_action(record);

    false
}

pub fn get_local(instruction: Instruction, data: &mut ThreadData) -> bool {
    let record = &mut data.call_stack.last_mut_unchecked().record;
    let GetLocal {
        destination,
        local_index,
    } = instruction.into();
    let local_register_index = record.get_local_register(local_index);
    let register = Register::Pointer(Pointer::Stack(local_register_index));

    record.set_register(destination, register);

    data.next_action = get_next_action(record);

    false
}

pub fn set_local(instruction: Instruction, data: &mut ThreadData) -> bool {
    let record = &mut data.call_stack.last_mut_unchecked().record;
    let SetLocal {
        register_index,
        local_index,
    } = instruction.into();
    let local_register_index = record.get_local_register(local_index);
    let register = Register::Pointer(Pointer::Stack(register_index));

    record.set_register(local_register_index, register);

    data.next_action = get_next_action(record);

    false
}

pub fn add(instruction: Instruction, data: &mut ThreadData) -> bool {
    let record = &mut data.call_stack.last_mut_unchecked().record;
    let Add {
        destination,
        left,
        right,
    } = instruction.into();
    let left = record.get_argument_unchecked(left);
    let right = record.get_argument_unchecked(right);
    let sum = left.add(right);
    let register = Register::Value(sum);

    record.set_register(destination, register);

    data.next_action = get_next_action(record);

    false
}

pub fn subtract(instruction: Instruction, data: &mut ThreadData) -> bool {
    let record = &mut data.call_stack.last_mut_unchecked().record;
    let Subtract {
        destination,
        left,
        right,
    } = instruction.into();
    let left = record.get_argument_unchecked(left);
    let right = record.get_argument_unchecked(right);
    let difference = left.subtract(right);
    let register = Register::Value(difference);

    record.set_register(destination, register);

    data.next_action = get_next_action(record);

    false
}

pub fn multiply(instruction: Instruction, data: &mut ThreadData) -> bool {
    let record = &mut data.call_stack.last_mut_unchecked().record;
    let Multiply {
        destination,
        left,
        right,
    } = instruction.into();
    let left = record.get_argument_unchecked(left);
    let right = record.get_argument_unchecked(right);
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

    data.next_action = get_next_action(record);

    false
}

pub fn divide(instruction: Instruction, data: &mut ThreadData) -> bool {
    let record = &mut data.call_stack.last_mut_unchecked().record;
    let Divide {
        destination,
        left,
        right,
    } = instruction.into();
    let left = record.get_argument_unchecked(left);
    let right = record.get_argument_unchecked(right);
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

    data.next_action = get_next_action(record);

    false
}

pub fn modulo(instruction: Instruction, data: &mut ThreadData) -> bool {
    let record = &mut data.call_stack.last_mut_unchecked().record;
    let Modulo {
        destination,
        left,
        right,
    } = instruction.into();
    let left = record.get_argument_unchecked(left);
    let right = record.get_argument_unchecked(right);
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

    data.next_action = get_next_action(record);

    false
}

pub fn test(instruction: Instruction, data: &mut ThreadData) -> bool {
    let record = &mut data.call_stack.last_mut_unchecked().record;
    let Test {
        operand_register,
        test_value,
    } = instruction.into();
    let value = record.open_register_unchecked(operand_register);
    let boolean = if let Value::Concrete(ConcreteValue::Boolean(boolean)) = value {
        *boolean
    } else {
        panic!("VM Error: Expected boolean value for TEST operation",);
    };

    if boolean == test_value {
        record.ip += 1;
    }

    data.next_action = get_next_action(record);

    false
}

pub fn test_set(instruction: Instruction, data: &mut ThreadData) -> bool {
    let record = &mut data.call_stack.last_mut_unchecked().record;
    let TestSet {
        destination,
        argument,
        test_value,
    } = instruction.into();
    let value = record.get_argument_unchecked(argument);
    let boolean = if let Value::Concrete(ConcreteValue::Boolean(boolean)) = value {
        *boolean
    } else {
        panic!("VM Error: Expected boolean value for TEST_SET operation",);
    };

    if boolean == test_value {
    } else {
        let pointer = match argument {
            Argument::Constant(constant_index) => Pointer::Constant(constant_index),
            Argument::Register(register_index) => Pointer::Stack(register_index),
        };
        let register = Register::Pointer(pointer);

        record.set_register(destination, register);
    }

    data.next_action = get_next_action(record);

    false
}

pub fn equal(instruction: Instruction, data: &mut ThreadData) -> bool {
    let record = &mut data.call_stack.last_mut_unchecked().record;
    let Equal { value, left, right } = instruction.into();
    let left = record.get_argument_unchecked(left);
    let right = record.get_argument_unchecked(right);
    let is_equal = left.equals(right);

    if is_equal == value {
        record.ip += 1;
    }

    data.next_action = get_next_action(record);

    false
}

pub fn less(instruction: Instruction, data: &mut ThreadData) -> bool {
    let record = &mut data.call_stack.last_mut_unchecked().record;
    let Less { value, left, right } = instruction.into();
    let left = record.get_argument_unchecked(left);
    let right = record.get_argument_unchecked(right);
    let is_less = left < right;

    if is_less == value {
        record.ip += 1;
    }

    data.next_action = get_next_action(record);

    false
}

pub fn less_equal(instruction: Instruction, data: &mut ThreadData) -> bool {
    let record = &mut data.call_stack.last_mut_unchecked().record;
    let LessEqual { value, left, right } = instruction.into();
    let left = record.get_argument_unchecked(left);
    let right = record.get_argument_unchecked(right);
    let is_less_or_equal = left <= right;

    if is_less_or_equal == value {
        record.ip += 1;
    }

    data.next_action = get_next_action(record);

    false
}

pub fn negate(instruction: Instruction, data: &mut ThreadData) -> bool {
    let record = &mut data.call_stack.last_mut_unchecked().record;
    let Negate {
        destination,
        argument,
    } = instruction.into();
    let argument = record.get_argument_unchecked(argument);
    let negated = argument.negate();
    let register = Register::Value(negated);

    record.set_register(destination, register);

    data.next_action = get_next_action(record);

    false
}

pub fn not(instruction: Instruction, data: &mut ThreadData) -> bool {
    let record = &mut data.call_stack.last_mut_unchecked().record;
    let Not {
        destination,
        argument,
    } = instruction.into();
    let argument = record.get_argument_unchecked(argument);
    let not = match argument {
        Value::Concrete(ConcreteValue::Boolean(boolean)) => ConcreteValue::Boolean(!boolean),
        _ => panic!("VM Error: Expected boolean value for NOT operation"),
    };
    let register = Register::Value(Value::Concrete(not));

    record.set_register(destination, register);

    data.next_action = get_next_action(record);

    false
}

pub fn jump(instruction: Instruction, data: &mut ThreadData) -> bool {
    let record = &mut data.call_stack.last_mut_unchecked().record;
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

    data.next_action = get_next_action(record);

    false
}

pub fn call(instruction: Instruction, data: &mut ThreadData) -> bool {
    let current_call = data.call_stack.pop_unchecked();
    let Call {
        destination: return_register,
        function_register,
        argument_count,
        is_recursive,
    } = instruction.into();
    let function = current_call
        .record
        .open_register_unchecked(function_register)
        .as_function()
        .unwrap();
    let first_argument_register = return_register - argument_count;
    let prototype = if is_recursive {
        current_call.record.chunk
    } else {
        &current_call.record.chunk.prototypes[function.prototype_index as usize]
    };
    let mut next_call = FunctionCall {
        name: prototype.name.clone(),
        return_register,
        ip: current_call.ip,
        record: Record::new(prototype),
    };

    for (argument_index, register_index) in (first_argument_register..return_register).enumerate() {
        let argument = current_call
            .record
            .clone_register_value_or_constant_unchecked(register_index);

        trace!(
            "Passing argument \"{argument}\" to {}",
            function
                .name
                .clone()
                .unwrap_or_else(|| DustString::from("anonymous"))
        );

        next_call
            .record
            .set_register(argument_index as u8, Register::Value(argument));
    }

    data.next_action = get_next_action(&mut next_call.record);

    data.call_stack.push(current_call);
    data.call_stack.push(next_call);

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
    let mut current_call = if data.call_stack.len() == 1 {
        if should_return_value {
            data.return_value_index = Some(return_register);
        };

        return true;
    } else {
        data.call_stack.pop_unchecked()
    };

    let outer_call = data.call_stack.last_mut_unchecked();
    let destination = current_call.return_register;

    if should_return_value {
        let return_value = current_call
            .record
            .empty_register_or_clone_constant_unchecked(return_register);

        outer_call
            .record
            .set_register(destination, Register::Value(return_value));
    }

    data.next_action = get_next_action(&mut outer_call.record);

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
