use std::ops::Range;

use rand::Rng;

use crate::{
    Value,
    vm::{Register, ThreadData, get_next_action},
};

pub fn random_int(data: &mut ThreadData, destination: u8, argument_range: Range<u8>) -> bool {
    let mut argument_range_iter = argument_range.into_iter();
    let (min, max) = {
        let mut min = None;

        loop {
            let register_index = argument_range_iter
                .next()
                .unwrap_or_else(|| panic!("No argument was passed to \"random_int\""));
            let value_option = data.open_register_allow_empty_unchecked(register_index);

            if let Some(argument) = value_option {
                if let Some(integer) = argument.as_integer() {
                    if min.is_none() {
                        min = Some(integer);
                    } else {
                        break (min, integer);
                    }
                }
            }
        }
    };

    let random_integer = rand::thread_rng().gen_range(min.unwrap()..max);

    data.set_register(destination, Register::Value(Value::integer(random_integer)));

    data.next_action = get_next_action(data);

    false
}
