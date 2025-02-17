use std::{ops::Range, panic};

use crate::vm::{RuntimeValue, Thread};

pub fn panic(data: &mut Thread, _: usize, argument_range: Range<usize>) {
    let current_frame = data.current_frame();
    let position = data.current_position();
    let mut message = format!("Dust panic at {position}!");

    for register_index in argument_range {
        let string_value = current_frame.get_string_from_register(register_index);

        match string_value {
            RuntimeValue::Raw(value) => {
                message.push_str(value.as_str());
            }
            RuntimeValue::Rc(rc) => {
                message.push_str(rc.as_str());
            }
            RuntimeValue::RefCell(ref_cell) => {
                message.push_str(ref_cell.borrow().as_str());
            }
        }

        message.push('\n');
    }

    panic!("{}", message)
}
