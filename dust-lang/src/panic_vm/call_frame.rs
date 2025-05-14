use std::sync::Arc;

use crate::{Address, Chunk};

use super::Memory;

#[derive(Debug)]
pub struct CallFrame {
    pub chunk: Arc<Chunk>,
    pub ip: usize,
    pub return_address: Address,
    pub memory: Memory,
}

impl CallFrame {
    pub fn new(chunk: Arc<Chunk>, return_address: Address) -> Self {
        CallFrame {
            memory: Memory::new(&chunk),
            chunk,
            ip: 0,
            return_address,
        }
    }
}
