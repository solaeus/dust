use std::{
    ops::Range,
    thread::{Builder, JoinHandle},
};

use tracing::{Level, info, span};

use crate::{DustString, instruction::InstructionBuilder, vm::Thread};

pub fn spawn(instruction: InstructionBuilder, thread: &mut Thread) {}
