use std::{ops::Range, panic};

use crate::vm::ThreadData;

pub fn panic(data: &mut ThreadData, _: Option<u8>, argument_range: Range<u8>) -> bool {
    let record = data.records.last_mut_unchecked();
    let position = record.current_position();
    let mut message = format!("Dust panic at {position}!");

    for register_index in argument_range {
        let value = record.open_register_unchecked(register_index);

        if let Some(string) = value.as_string() {
            message.push_str(string);
            message.push('\n');
        }
    }

    panic!("{}", message)
}
