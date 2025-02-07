use std::{ops::Range, thread::JoinHandle};

use crate::vm::Thread;

fn start_thread(_thread: &mut Thread, _argument_range: Range<usize>) -> JoinHandle<()> {
    todo!();
}

pub fn spawn(data: &mut Thread, _: usize, argument_range: Range<usize>) {
    let _ = start_thread(data, argument_range);
}
