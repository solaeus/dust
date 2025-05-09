use std::sync::Arc;

use crate::Chunk;

#[derive(Clone, Debug)]
pub struct CallFrame {
    pub chunk: Arc<Chunk>,
    pub ip: usize,
    pub return_register: u16,
}

impl CallFrame {
    pub fn new(chunk: Arc<Chunk>, return_register: u16) -> Self {
        CallFrame {
            chunk,
            ip: 0,
            return_register,
        }
    }
}
