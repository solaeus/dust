use std::sync::Arc;

use crate::{Chunk, Destination};

#[derive(Debug)]
pub struct CallFrame {
    pub chunk: Arc<Chunk>,
    pub ip: usize,
    pub return_address: Destination,
}

impl CallFrame {
    pub fn new(chunk: Arc<Chunk>, return_address: Destination) -> Self {
        CallFrame {
            chunk,
            ip: 0,
            return_address,
        }
    }
}
