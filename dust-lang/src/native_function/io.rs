use std::io::{Write, stdin, stdout};
use std::ops::Range;

use crate::{
    ConcreteValue, Value,
    vm::{Register, ThreadData, get_next_action},
};

pub fn read_line(data: &mut ThreadData, destination: u8, _argument_range: Range<u8>) -> bool {
    let mut buffer = String::new();

    if stdin().read_line(&mut buffer).is_ok() {
        let length = buffer.len();

        buffer.truncate(length.saturating_sub(1));

        let register = Register::Value(Value::Concrete(ConcreteValue::string(buffer)));

        data.set_register(destination, register);
    }

    data.next_action = get_next_action(data);

    false
}

pub fn write(data: &mut ThreadData, _: u8, argument_range: Range<u8>) -> bool {
    let mut stdout = stdout();

    for register_index in argument_range {
        if let Some(value) = data.open_register_allow_empty_unchecked(register_index) {
            let string = value.display(data);
            let _ = stdout.write(string.as_bytes());
        }
    }

    let _ = stdout.flush();
    data.next_action = get_next_action(data);

    false
}

pub fn write_line(data: &mut ThreadData, _: u8, argument_range: Range<u8>) -> bool {
    let mut stdout = stdout().lock();

    for register_index in argument_range {
        if let Some(value) = data.open_register_allow_empty_unchecked(register_index) {
            let string = value.display(data);
            let _ = stdout.write(string.as_bytes());
            let _ = stdout.write(b"\n");
        }
    }

    let _ = stdout.flush();
    data.next_action = get_next_action(data);

    false
}
