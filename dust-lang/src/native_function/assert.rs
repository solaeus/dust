use std::{ops::Range, panic};

use crate::vm::Thread;

pub fn panic(data: &mut Thread, _: usize, argument_range: Range<usize>) {
    let current_frame = data.current_frame();
    let position = data.current_position();
    let mut message = format!("Dust panic at {position}!");

    for register_index in argument_range {
        let string = current_frame
            .registers
            .strings
            .get(register_index)
            .as_value();

        message.push_str(string);
        message.push('\n');
    }

    panic!("{}", message)
}
