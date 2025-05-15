use crate::{Address, Chunk};

#[derive(Debug)]
pub struct CallFrame<'a> {
    pub chunk: &'a Chunk,
    pub ip: usize,
    pub return_address: Address,
}

impl<'a> CallFrame<'a> {
    pub fn new(chunk: &'a Chunk, return_address: Address) -> Self {
        CallFrame {
            chunk,
            ip: 0,
            return_address,
        }
    }
}
