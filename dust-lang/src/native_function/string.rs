use std::ops::Range;

use crate::{
    ConcreteValue, Value,
    vm::{Register, Thread},
};

pub fn to_string(thread: &mut Thread, destination: usize, argument_range: Range<usize>) {
    let argument_value = thread.get_register(argument_range.start);
    let argument_string = argument_value.display(thread);
    let new_register = Register::Value(Value::Concrete(ConcreteValue::string(argument_string)));
    let old_register = thread.get_register_mut(destination);

    *old_register = new_register;
}
