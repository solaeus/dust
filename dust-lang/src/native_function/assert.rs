use std::{ops::Range, panic};

use crate::vm::Thread;

pub fn panic(data: &mut Thread, _: usize, argument_range: Range<usize>) {
    let position = data.current_position();
    let mut message = format!("Dust panic at {position}!");

    for register_index in argument_range {
        let value = data.get_register(register_index);
        let string = value.display(data);

        message.push_str(&string);
        message.push('\n');
    }

    panic!("{}", message)
}
