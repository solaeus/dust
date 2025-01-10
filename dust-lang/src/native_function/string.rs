use std::ops::Range;

use crate::{
    ConcreteValue, Value,
    vm::{Register, ThreadData, get_next_action},
};

pub fn to_string(data: &mut ThreadData, destination: u8, argument_range: Range<u8>) -> bool {
    let argument_value = data.open_register_unchecked(argument_range.start);
    let argument_string = argument_value.display(data);
    let register = Register::Value(Value::Concrete(ConcreteValue::string(argument_string)));

    data.set_register(destination, register);

    data.next_action = get_next_action(data);

    false
}
