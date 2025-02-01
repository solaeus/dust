use std::{ops::Range, panic};

use crate::{instruction::InstructionBuilder, vm::Thread};

pub fn panic(instruction: InstructionBuilder, thread: &mut Thread) {}
