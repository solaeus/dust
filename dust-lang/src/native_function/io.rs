use std::io::{Write, stdin, stdout};
use std::ops::Range;

use crate::{
    Value,
    instruction::InstructionBuilder,
    vm::{Register, Thread},
};

pub fn read_line(instruction: InstructionBuilder, thread: &mut Thread) {}

pub fn write(instruction: InstructionBuilder, thread: &mut Thread) {}

pub fn write_line(instruction: InstructionBuilder, thread: &mut Thread) {}
