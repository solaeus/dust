use std::{ops::Range, panic};

use crate::vm::Thread;

pub fn panic(data: &mut Thread, _: usize, argument_range: Range<usize>) {
    let current_frame = data.current_frame();
    let position = data.current_position();
    let mut message = format!("Dust panic at {position}!");

    for register_index in argument_range {
        let string = current_frame.get_string_from_register(register_index);

        message.push_str(string);
        message.push('\n');
    }

    panic!("{}", message)
}
