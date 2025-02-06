use std::io::{Write, stdin, stdout};
use std::ops::Range;

use crate::DustString;
use crate::vm::Thread;

pub fn read_line(data: &mut Thread, destination: usize, _argument_range: Range<usize>) {
    let mut buffer = String::new();

    if stdin().read_line(&mut buffer).is_ok() {
        let length = buffer.len();

        buffer.truncate(length.saturating_sub(1));

        let string = DustString::from(buffer);

        data.set_string_register(destination, string);
    }
}

pub fn write(data: &mut Thread, _: usize, argument_range: Range<usize>) {
    let mut stdout = stdout();

    for register_index in argument_range {
        let value = data.get_string_register(register_index);
        let _ = stdout.write(value.to_string().as_bytes());
    }

    let _ = stdout.flush();
}

pub fn write_line(data: &mut Thread, _: usize, argument_range: Range<usize>) {
    let mut stdout = stdout().lock();

    for register_index in argument_range {
        let value = data.get_string_register(register_index);
        let _ = stdout.write(value.to_string().as_bytes());
    }

    let _ = stdout.write(b"\n");
    let _ = stdout.flush();
}
