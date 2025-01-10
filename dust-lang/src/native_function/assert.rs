use std::{ops::Range, panic};

use crate::vm::ThreadData;

pub fn panic(data: &mut ThreadData, _: u8, argument_range: Range<u8>) -> bool {
    let position = data.current_position();
    let mut message = format!("Dust panic at {position}!");

    for register_index in argument_range {
        let value_option = data.open_register_allow_empty_unchecked(register_index);
        let value = match value_option {
            Some(value) => value,
            None => continue,
        };
        let string = value.display(data);

        message.push_str(&string);
        message.push('\n');
    }

    panic!("{}", message)
}
