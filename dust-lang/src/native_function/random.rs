use std::ops::Range;

use rand::Rng;

use crate::vm::{Register, Thread};

pub fn random_int(data: &mut Thread, destination: usize, argument_range: Range<usize>) {
    let mut argument_range_iter = argument_range.into_iter();
    let (min, max) = {
        let mut min = None;

        loop {
            let register_index = argument_range_iter
                .next()
                .unwrap_or_else(|| panic!("No argument was passed to \"random_int\""));
            let integer = data.get_integer_register(register_index);

            if min.is_none() {
                min = Some(*integer);
            } else {
                break (min, *integer);
            }
        }
    };

    let random_integer = rand::thread_rng().gen_range(min.unwrap()..max);
    let register = Register::Value(random_integer);

    data.set_integer_register(destination, register);
}
