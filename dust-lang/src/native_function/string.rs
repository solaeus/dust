use std::ops::Range;

use crate::{
    vm::{get_next_action, Record, Register, ThreadSignal},
    ConcreteValue, NativeFunctionError, Value,
};

pub fn to_string(
    record: &mut Record,
    destination: Option<u8>,
    argument_range: Range<u8>,
) -> Result<ThreadSignal, NativeFunctionError> {
    let argument_value = record.open_register_unchecked(argument_range.start);
    let argument_string = argument_value.display(record);
    let destination = destination.unwrap();
    let register = Register::Value(Value::Concrete(ConcreteValue::string(argument_string)));

    record.set_register(destination, register);

    let next_action = get_next_action(record);

    Ok(ThreadSignal::Continue(next_action))
}
