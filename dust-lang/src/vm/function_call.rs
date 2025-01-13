use std::{
    fmt::{self, Debug, Display, Formatter},
    sync::Arc,
};

use crate::{Chunk, DustString};

use super::Register;

#[derive(Debug)]
pub struct FunctionCall {
    pub chunk: Arc<Chunk>,
    pub ip: usize,
    pub return_register: u16,
    pub registers: Vec<Register>,
}

impl FunctionCall {
    pub fn new(chunk: Arc<Chunk>, return_register: u16) -> Self {
        let register_count = chunk.register_count;

        Self {
            chunk,
            ip: 0,
            return_register,
            registers: vec![Register::Empty; register_count],
        }
    }
}

impl Display for FunctionCall {
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
