use std::ops::Range;

use crate::{
    vm::{get_next_action, Register, ThreadData},
    ConcreteValue, Value,
};

pub fn to_string(
    data: &mut ThreadData,
    destination: Option<u8>,
    argument_range: Range<u8>,
) -> bool {
    let record = data.records.last_mut_unchecked();
    let argument_value = record.open_register_unchecked(argument_range.start);
    let argument_string = argument_value.display(record);
    let destination = destination.unwrap();
    let register = Register::Value(Value::Concrete(ConcreteValue::string(argument_string)));

    record.set_register(destination, register);

    data.next_action = get_next_action(record);

    false
}
