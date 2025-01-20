use std::ops::Range;

use crate::{
    Value,
    vm::{Register, ThreadData},
};

pub fn to_string(data: &mut ThreadData, destination: u16, argument_range: Range<u16>) -> bool {
    todo!()

    // let argument_value = data.open_register_unchecked(argument_range.start);
    // let argument_string = argument_value.display(data);
    // let register = Register::Value(Value::Concrete(ConcreteValue::string(argument_string)));

    // data.set_register(destination, register);

    // data.next_action = get_next_action(data);

    // false
}
