use std::io::{stdin, stdout, Write};
use std::ops::Range;

use crate::vm::Thread;
use crate::DustString;

pub fn read_line(data: &mut Thread, destination: usize, _argument_range: Range<usize>) {
    let current_frame = data.current_frame_mut();
    let mut buffer = String::new();

    if stdin().read_line(&mut buffer).is_ok() {
        let length = buffer.len();

        buffer.truncate(length.saturating_sub(1));

        let string = DustString::from(buffer);

        current_frame
            .registers
            .strings
            .set_to_new_register(destination, string);
    }
}

pub fn write(data: &mut Thread, _: usize, argument_range: Range<usize>) {
    let current_frame = data.current_frame_mut();
    let mut stdout = stdout();

    for register_index in argument_range {
        let string = current_frame
            .registers
            .strings
            .get(register_index)
            .as_value();
        let _ = stdout.write(string.as_bytes());
    }

    let _ = stdout.flush();
}

pub fn write_line(data: &mut Thread, _: usize, argument_range: Range<usize>) {
    let current_frame = data.current_frame_mut();
    let mut stdout = stdout().lock();

    for register_index in argument_range {
        let string = current_frame
            .registers
            .strings
            .get(register_index)
            .as_value();
        let _ = stdout.write(string.as_bytes());
    }

    let _ = stdout.write(b"\n");
    let _ = stdout.flush();
}
