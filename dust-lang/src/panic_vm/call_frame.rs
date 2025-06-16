use std::sync::Arc;

use crate::Address;

#[derive(Debug)]
pub struct CallFrame<C> {
    pub chunk: Arc<C>,
    pub ip: usize,
    pub return_address: Address,
}

impl<C> CallFrame<C> {
    pub fn new(chunk: Arc<C>, return_address: Address) -> Self {
        CallFrame {
            chunk,
            ip: 0,
            return_address,
        }
    }
}
