use std::panic;

use smallvec::SmallVec;

use crate::{vm::Record, DustString, NativeFunctionError, Value};

pub fn panic(
    record: &mut Record,
    arguments: SmallVec<[&Value; 4]>,
) -> Result<Option<Value>, NativeFunctionError> {
    let mut message: Option<DustString> = None;

    for value_ref in arguments {
        let string = value_ref.display(record);

        match message {
            Some(ref mut message) => message.push_str(&string),
            None => message = Some(string),
        }
    }

    if let Some(message) = message {
        panic!("{message}");
    } else {
        panic!("Explicit panic");
    }
}
