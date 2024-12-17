use std::{ops::Range, panic};

use crate::{
    vm::{Record, ThreadSignal},
    NativeFunctionError,
};

pub fn panic(
    record: &mut Record,
    _: Option<u8>,
    argument_range: Range<u8>,
) -> Result<ThreadSignal, NativeFunctionError> {
    let position = record.current_position();
    let mut message = format!("Dust panic at {position}!");

    for register_index in argument_range {
        let value = record.open_register(register_index);

        if let Some(string) = value.as_string() {
            message.push_str(string);
            message.push('\n');
        }
    }

    panic!("{}", message)
}
