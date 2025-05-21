use std::sync::Arc;

use crate::{Address, Chunk};

#[derive(Debug)]
pub struct CallFrame {
    pub chunk: Arc<Chunk>,
    pub ip: usize,
    pub return_address: Address,
}

impl CallFrame {
    pub fn new(chunk: Arc<Chunk>, return_address: Address) -> Self {
        CallFrame {
            chunk,
            ip: 0,
            return_address,
        }
    }
}
