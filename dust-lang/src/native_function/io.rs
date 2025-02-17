use std::io::{stdin, stdout, Write};
use std::ops::Range;

use crate::vm::{Register, RuntimeValue, Thread};
use crate::DustString;

pub fn read_line(data: &mut Thread, destination: usize, _argument_range: Range<usize>) {
    let current_frame = data.current_frame_mut();
    let mut buffer = String::new();

    if stdin().read_line(&mut buffer).is_ok() {
        let length = buffer.len();

        buffer.truncate(length.saturating_sub(1));

        let string = RuntimeValue::Raw(DustString::from(buffer));

        current_frame.registers.strings[destination].set(string);
    }
}

pub fn write(data: &mut Thread, _: usize, argument_range: Range<usize>) {
    let current_frame = data.current_frame();
    let mut stdout = stdout();

    for register_index in argument_range {
        let value = current_frame.get_string_from_register(register_index);

        match value {
            RuntimeValue::Raw(value) => {
                let _ = stdout.write(value.as_bytes());
            }
            RuntimeValue::Rc(value) => {
                let _ = stdout.write(value.as_bytes());
            }
            RuntimeValue::RefCell(ref_cell) => {
                let _ = stdout.write(ref_cell.borrow().as_bytes());
            }
        }
    }

    let _ = stdout.flush();
}

pub fn write_line(data: &mut Thread, _: usize, argument_range: Range<usize>) {
    let current_frame = data.current_frame();
    let mut stdout = stdout().lock();

    for register_index in argument_range {
        let value = current_frame.get_string_from_register(register_index);

        match value {
            RuntimeValue::Raw(value) => {
                let _ = stdout.write(value.as_bytes());
            }
            RuntimeValue::Rc(value) => {
                let _ = stdout.write(value.as_bytes());
            }
            RuntimeValue::RefCell(ref_cell) => {
                let _ = stdout.write(ref_cell.borrow().as_bytes());
            }
        }
    }

    let _ = stdout.write(b"\n");
    let _ = stdout.flush();
}
