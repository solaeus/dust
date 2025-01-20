use std::io::{Write, stdin, stdout};
use std::ops::Range;

use crate::DustString;
use crate::{
    Value,
    vm::{Register, ThreadData},
};

pub fn read_line(data: &mut ThreadData, destination: u16, _argument_range: Range<u16>) -> bool {
    let current_frame = data.current_frame_mut();
    let mut buffer = String::new();

    if stdin().read_line(&mut buffer).is_ok() {
        let length = buffer.len();

        buffer.truncate(length.saturating_sub(1));

        let register = current_frame.registers.get_string_mut(destination);

        *register = Register::Value(DustString::from(buffer));
    }

    false
}

pub fn write(data: &mut ThreadData, _: u16, argument_range: Range<u16>) -> bool {
    todo!()
}

pub fn write_line(data: &mut ThreadData, _: u16, argument_range: Range<u16>) -> bool {
    todo!()
}
