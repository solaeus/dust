use std::fmt::{self, Debug, Display, Formatter};

use crate::{Chunk, DustString};

use super::Register;

#[derive(Debug)]
pub struct FunctionCall<'a> {
    pub chunk: &'a Chunk,
    pub ip: usize,
    pub return_register: u8,
    pub registers: Vec<Register>,
}

impl<'a> FunctionCall<'a> {
    pub fn new(chunk: &'a Chunk, return_register: u8) -> Self {
        Self {
            chunk,
            ip: 0,
            return_register,
            registers: vec![Register::Empty; chunk.register_count],
        }
    }
}

impl Display for FunctionCall<'_> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(
            f,
            "FunctionCall: {} | IP: {} | Registers: {}",
            self.chunk
                .name
                .as_ref()
                .unwrap_or(&DustString::from("anonymous")),
            self.ip,
            self.registers.len()
        )
    }
}
