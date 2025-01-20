use std::{ops::Range, panic};

use crate::vm::ThreadData;

pub fn panic(data: &mut ThreadData, _: u16, argument_range: Range<u16>) -> bool {
    todo!()
}
