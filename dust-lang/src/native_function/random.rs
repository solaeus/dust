use std::ops::Range;

use rand::Rng;

use crate::vm::Thread;

pub fn random_int(data: &mut Thread, destination: usize, argument_range: Range<usize>) {
    let current_frame = data.current_frame_mut();
    let mut argument_range_iter = argument_range.into_iter();
    let (min, max) = {
        let mut min = None;

        loop {
            let register_index = argument_range_iter
                .next()
                .unwrap_or_else(|| panic!("No argument was passed to \"random_int\""));
            let integer = current_frame.get_integer_from_register(register_index);

            if min.is_none() {
                min = Some(integer);
            } else {
                break (min, integer);
            }
        }
    };

    let random_integer = rand::thread_rng().gen_range(min.unwrap()..max);

    current_frame
        .registers
        .integers
        .set_to_new_register(destination, random_integer);
}
