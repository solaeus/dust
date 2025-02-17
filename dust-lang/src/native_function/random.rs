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
            let integer = current_frame
                .registers
                .integers
                .get(register_index)
                .copy_value();

            if let Some(min) = min {
                break (min, integer);
            } else {
                min = Some(integer);
            }
        }
    };

    let random_integer = rand::thread_rng().gen_range(min..max);

    current_frame
        .registers
        .integers
        .set_to_new_register(destination, random_integer);
}
