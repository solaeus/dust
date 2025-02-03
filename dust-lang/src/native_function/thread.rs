use std::{
    ops::Range,
    thread::{Builder, JoinHandle},
};

use tracing::{Level, info, span};

use crate::{DustString, vm::Thread};

fn start_thread(thread: &mut Thread, argument_range: Range<usize>) -> JoinHandle<()> {
    todo!();
}

pub fn spawn(data: &mut Thread, _: usize, argument_range: Range<usize>) {
    let _ = start_thread(data, argument_range);
}
