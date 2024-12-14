use std::{borrow::BorrowMut, cell::RefCell, rc::Rc};

use smallvec::SmallVec;

use crate::{AbstractValue, ConcreteValue, NativeFunction, Type, Value};

use super::{Instruction, InstructionData, Pointer, Register, Vm};

#[derive(Clone, Copy, Debug)]
pub struct Runner {
    logic: RunnerLogic,
    data: InstructionData,
}

impl Runner {
    pub fn new(instruction: Instruction) -> Self {
        let (operation, data) = instruction.decode();
        let logic = RUNNER_LOGIC_TABLE[operation.0 as usize];

        Self { logic, data }
    }

    pub fn from_parts(logic: RunnerLogic, data: InstructionData) -> Self {
        Self { logic, data }
    }

    pub fn run(&self, vm: &mut Vm) {
        (self.logic)(vm, self.data);
    }
}

pub type RunnerLogic = fn(&mut Vm, InstructionData);

pub const RUNNER_LOGIC_TABLE: [RunnerLogic; 24] = [
    r#move,
    close,
    load_boolean,
    load_constant,
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

pub fn r#move(vm: &mut Vm, instruction_data: InstructionData) {
    let InstructionData { b, c, .. } = instruction_data;
    let from_register_has_value = vm
        .stack
        .get(b as usize)
        .is_some_and(|register| !matches!(register, Register::Empty));
    let register = Register::Pointer(Pointer::Stack(b));

    if from_register_has_value {
        vm.set_register(c, register);
    }

    vm.ip += 1;

    vm.execute_next_runner();
}

#[allow(clippy::needless_lifetimes)]
pub fn close<'b, 'c>(vm: &'b mut Vm<'c>, instruction_data: InstructionData) {
    let InstructionData { b, c, .. } = instruction_data;

    assert!(b < c, "Runtime Error: Malformed instruction");

    for register_index in b..c {
        assert!(
            (register_index as usize) < vm.stack.len(),
            "Runtime Error: Register index out of bounds"
        );

        vm.stack[register_index as usize] = Register::Empty;
    }

    vm.ip += 1;

    vm.execute_next_runner();
}

#[allow(clippy::needless_lifetimes)]
pub fn load_boolean<'b, 'c>(vm: &'b mut Vm<'c>, instruction_data: InstructionData) {
    let InstructionData { a, b, c, .. } = instruction_data;
    let boolean = ConcreteValue::Boolean(b != 0).to_value();
    let register = Register::Value(boolean);

    vm.set_register(a, register);

    if c != 0 {
        vm.ip += 2;
    } else {
        vm.ip += 1;
    }

    vm.execute_next_runner();
}

#[allow(clippy::needless_lifetimes)]
pub fn load_constant<'b, 'c>(vm: &'b mut Vm<'c>, instruction_data: InstructionData) {
    let InstructionData { a, b, c, .. } = instruction_data;
    let register = Register::Pointer(Pointer::Constant(b));

    vm.set_register(a, register);

    if c != 0 {
        vm.ip += 2;
    } else {
        vm.ip += 1;
    }

    vm.execute_next_runner();
}

#[allow(clippy::needless_lifetimes)]
pub fn load_list<'b, 'c>(vm: &'b mut Vm<'c>, instruction_data: InstructionData) {
    let InstructionData { a, b, .. } = instruction_data;
    let mut item_pointers = Vec::with_capacity((a - b) as usize);
    let stack = vm.stack.as_slice();
    let mut item_type = Type::Any;

    for register_index in b..a {
        match &stack[register_index as usize] {
            Register::Empty => continue,
            Register::Value(value) => {
                if item_type == Type::Any {
                    item_type = value.r#type();
                }
            }
            Register::Pointer(pointer) => {
                if item_type == Type::Any {
                    item_type = vm.follow_pointer(*pointer).r#type();
                }
            }
        }

        let pointer = Pointer::Stack(register_index);

        item_pointers.push(pointer);
    }

    let list_value = AbstractValue::List {
        item_type,
        item_pointers,
    }
    .to_value();
    let register = Register::Value(list_value);

    vm.set_register(a, register);

    vm.ip += 1;

    vm.execute_next_runner();
}

#[allow(clippy::needless_lifetimes)]
pub fn load_self<'b, 'c>(vm: &'b mut Vm<'c>, instruction_data: InstructionData) {
    let InstructionData { a, .. } = instruction_data;
    let register = Register::Value(AbstractValue::FunctionSelf.to_value());

    vm.set_register(a, register);

    vm.ip += 1;

    vm.execute_next_runner();
}

#[allow(clippy::needless_lifetimes)]
pub fn get_local<'b, 'c>(vm: &'b mut Vm<'c>, instruction_data: InstructionData) {
    let InstructionData { a, b, .. } = instruction_data;
    let local_register_index = vm.get_local_register(b);
    let register = Register::Pointer(Pointer::Stack(local_register_index));

    vm.set_register(a, register);

    vm.ip += 1;

    vm.execute_next_runner();
}

#[allow(clippy::needless_lifetimes)]
pub fn set_local<'b, 'c>(vm: &'b mut Vm<'c>, instruction_data: InstructionData) {
    let InstructionData { b, c, .. } = instruction_data;
    let local_register_index = vm.get_local_register(c);
    let register = Register::Pointer(Pointer::Stack(b));

    vm.set_register(local_register_index, register);

    vm.ip += 1;

    vm.execute_next_runner();
}

#[allow(clippy::needless_lifetimes)]
pub fn add<'b, 'c>(vm: &'b mut Vm<'c>, instruction_data: InstructionData) {
    let InstructionData {
        a,
        b,
        c,
        b_is_constant,
        c_is_constant,
        ..
    } = instruction_data;
    let left = vm.get_argument(b, b_is_constant);
    let right = vm.get_argument(c, c_is_constant);
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

    vm.set_register(a, register);

    vm.ip += 1;

    vm.execute_next_runner();
}

#[allow(clippy::needless_lifetimes)]
pub fn subtract<'b, 'c>(vm: &'b mut Vm<'c>, instruction_data: InstructionData) {
    let InstructionData {
        a,
        b,
        c,
        b_is_constant,
        c_is_constant,
        ..
    } = instruction_data;
    let left = vm.get_argument(b, b_is_constant);
    let right = vm.get_argument(c, c_is_constant);
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

    vm.set_register(a, register);

    vm.ip += 1;

    vm.execute_next_runner();
}

#[allow(clippy::needless_lifetimes)]
pub fn multiply<'b, 'c>(vm: &'b mut Vm<'c>, instruction_data: InstructionData) {
    let InstructionData {
        a,
        b,
        c,
        b_is_constant,
        c_is_constant,
        ..
    } = instruction_data;
    let left = vm.get_argument(b, b_is_constant);
    let right = vm.get_argument(c, c_is_constant);
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

    vm.set_register(a, register);

    vm.ip += 1;

    vm.execute_next_runner();
}

#[allow(clippy::needless_lifetimes)]
pub fn divide<'b, 'c>(vm: &'b mut Vm<'c>, instruction_data: InstructionData) {
    let InstructionData {
        a,
        b,
        c,
        b_is_constant,
        c_is_constant,
        ..
    } = instruction_data;
    let left = vm.get_argument(b, b_is_constant);
    let right = vm.get_argument(c, c_is_constant);
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

    vm.set_register(a, register);

    vm.ip += 1;

    vm.execute_next_runner();
}

#[allow(clippy::needless_lifetimes)]
pub fn modulo<'b, 'c>(vm: &'b mut Vm<'c>, instruction_data: InstructionData) {
    let InstructionData {
        a,
        b,
        c,
        b_is_constant,
        c_is_constant,
        ..
    } = instruction_data;
    let left = vm.get_argument(b, b_is_constant);
    let right = vm.get_argument(c, c_is_constant);
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

    vm.set_register(a, register);

    vm.ip += 1;

    vm.execute_next_runner();
}

#[allow(clippy::needless_lifetimes)]
pub fn test<'b, 'c>(vm: &'b mut Vm<'c>, instruction_data: InstructionData) {
    let InstructionData {
        b,
        b_is_constant,
        c,
        ..
    } = instruction_data;
    let value = vm.get_argument(b, b_is_constant);
    let boolean = if let Value::Concrete(ConcreteValue::Boolean(boolean)) = value {
        *boolean
    } else {
        panic!(
            "VM Error: Expected boolean value for TEST operation at {}",
            vm.current_position()
        );
    };
    let test_value = c != 0;

    if boolean == test_value {
        vm.ip += 2;
    } else {
        vm.ip += 1;
    }

    vm.execute_next_runner();
}

#[allow(clippy::needless_lifetimes)]
pub fn test_set<'b, 'c>(vm: &'b mut Vm<'c>, instruction_data: InstructionData) {
    let InstructionData {
        a,
        b,
        c,
        b_is_constant,
        ..
    } = instruction_data;
    let value = vm.get_argument(b, b_is_constant);
    let boolean = if let Value::Concrete(ConcreteValue::Boolean(boolean)) = value {
        *boolean
    } else {
        panic!(
            "VM Error: Expected boolean value for TEST_SET operation at {}",
            vm.current_position()
        );
    };
    let test_value = c != 0;

    if boolean == test_value {
        vm.ip += 2;
    } else {
        let pointer = if b_is_constant {
            Pointer::Constant(b)
        } else {
            Pointer::Stack(b)
        };
        let register = Register::Pointer(pointer);

        vm.set_register(a, register);

        vm.ip += 1;
    }

    vm.execute_next_runner();
}

#[allow(clippy::needless_lifetimes)]
pub fn equal<'b, 'c>(vm: &'b mut Vm<'c>, instruction_data: InstructionData) {
    let InstructionData {
        b,
        c,
        b_is_constant,
        c_is_constant,
        d,
        ..
    } = instruction_data;
    let left = vm.get_argument(b, b_is_constant);
    let right = vm.get_argument(c, c_is_constant);
    let is_equal = left == right;

    if is_equal == d {
        vm.ip += 2;
    } else {
        vm.ip += 1;
    }

    vm.execute_next_runner();
}

pub fn less(vm: &mut Vm, instruction_data: InstructionData) {
    let InstructionData {
        b,
        c,
        b_is_constant,
        c_is_constant,
        d,
        ..
    } = instruction_data;
    let left = vm.get_argument(b, b_is_constant);
    let right = vm.get_argument(c, c_is_constant);
    let is_less = left < right;

    if is_less == d {
        vm.ip += 2;
    } else {
        vm.ip += 1;
    }

    vm.execute_next_runner();
}

#[allow(clippy::needless_lifetimes)]
pub fn less_equal<'b, 'c>(vm: &'b mut Vm<'c>, instruction_data: InstructionData) {
    let InstructionData {
        b,
        c,
        b_is_constant,
        c_is_constant,
        d,
        ..
    } = instruction_data;
    let left = vm.get_argument(b, b_is_constant);
    let right = vm.get_argument(c, c_is_constant);
    let is_less_or_equal = left <= right;

    if is_less_or_equal == d {
        vm.ip += 2;
    } else {
        vm.ip += 1;
    }

    vm.execute_next_runner();
}

#[allow(clippy::needless_lifetimes)]
pub fn negate<'b, 'c>(vm: &'b mut Vm<'c>, instruction_data: InstructionData) {
    let InstructionData {
        a,
        b,
        b_is_constant,
        ..
    } = instruction_data;
    let argument = vm.get_argument(b, b_is_constant);
    let negated = match argument {
        Value::Concrete(value) => match value {
            ConcreteValue::Float(float) => ConcreteValue::Float(-float),
            ConcreteValue::Integer(integer) => ConcreteValue::Integer(-integer),
            _ => panic!("Value Error: Cannot negate value"),
        },
        Value::Abstract(_) => panic!("VM Error: Cannot negate value"),
    };
    let register = Register::Value(Value::Concrete(negated));

    vm.set_register(a, register);

    vm.ip += 1;

    vm.execute_next_runner();
}

#[allow(clippy::needless_lifetimes)]
pub fn not<'b, 'c>(vm: &'b mut Vm<'c>, instruction_data: InstructionData) {
    let InstructionData {
        a,
        b,
        b_is_constant,
        ..
    } = instruction_data;
    let argument = vm.get_argument(b, b_is_constant);
    let not = match argument {
        Value::Concrete(ConcreteValue::Boolean(boolean)) => ConcreteValue::Boolean(!boolean),
        _ => panic!("VM Error: Expected boolean value for NOT operation"),
    };
    let register = Register::Value(Value::Concrete(not));

    vm.set_register(a, register);

    vm.ip += 1;

    vm.execute_next_runner();
}

#[allow(clippy::needless_lifetimes)]
pub fn jump<'c>(vm: &mut Vm<'c>, instruction_data: InstructionData) {
    let InstructionData { b, c, .. } = instruction_data;
    let offset = b as usize;
    let is_positive = c != 0;

    if is_positive {
        vm.ip += offset + 1
    } else {
        vm.ip -= offset
    }

    vm.execute_next_runner();
}

#[allow(clippy::needless_lifetimes)]
pub fn call<'b, 'c>(vm: &'b mut Vm<'c>, instruction_data: InstructionData) {
    let InstructionData {
        a,
        b,
        c,
        b_is_constant,
        ..
    } = instruction_data;
    let function = vm.get_argument(b, b_is_constant);
    let mut function_vm = if let Value::Concrete(ConcreteValue::Function(chunk)) = function {
        Vm::new(vm.source, chunk, Some(vm), None)
    } else if let Value::Abstract(AbstractValue::FunctionSelf) = function {
        Vm::new(vm.source, vm.chunk, Some(vm), Some(vm.runners.clone()))
    } else {
        panic!("VM Error: Expected function")
    };
    let first_argument_index = a - c;
    let mut argument_index = 0;

    for argument_register_index in first_argument_index..a {
        let target_register_is_empty =
            matches!(vm.stack[argument_register_index as usize], Register::Empty);

        if target_register_is_empty {
            continue;
        }

        function_vm.set_register(
            argument_index as u8,
            Register::Pointer(Pointer::ParentStack(argument_register_index)),
        );

        argument_index += 1;
    }

    let return_value = function_vm.run();

    if let Some(concrete_value) = return_value {
        let register = Register::Value(concrete_value);

        vm.set_register(a, register);
    }

    vm.ip += 1;

    vm.execute_next_runner();
}

#[allow(clippy::needless_lifetimes)]
pub fn call_native<'b, 'c>(vm: &'b mut Vm<'c>, instruction_data: InstructionData) {
    let InstructionData { a, b, c, .. } = instruction_data;
    let first_argument_index = (a - c) as usize;
    let argument_range = first_argument_index..a as usize;
    let mut arguments: SmallVec<[&Value; 4]> = SmallVec::new();

    for register_index in argument_range {
        let register = &vm.stack[register_index];
        let value = match register {
            Register::Value(value) => value,
            Register::Pointer(pointer) => {
                let value_option = vm.follow_pointer_allow_empty(*pointer);

                match value_option {
                    Some(value) => value,
                    None => continue,
                }
            }
            Register::Empty => continue,
        };

        arguments.push(value);
    }

    let function = NativeFunction::from(b);
    let return_value = function.call(vm, arguments).unwrap();

    if let Some(value) = return_value {
        let register = Register::Value(value);

        vm.set_register(a, register);
    }

    vm.ip += 1;

    vm.execute_next_runner();
}

#[allow(clippy::needless_lifetimes)]
pub fn r#return<'b, 'c>(vm: &'b mut Vm<'c>, instruction_data: InstructionData) {
    let should_return_value = instruction_data.b != 0;

    if !should_return_value {
        return vm.ip += 1;
    }

    if let Some(register_index) = &vm.last_assigned_register {
        let return_value = vm.open_register(*register_index).clone();

        vm.return_value = Some(return_value);
    } else {
        panic!("Stack underflow");
    }
}
