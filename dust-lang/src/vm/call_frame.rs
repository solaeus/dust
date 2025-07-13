use std::sync::Arc;

use crate::{Address, OperandType, StrippedChunk};

#[derive(Clone, Debug)]
pub struct CallFrame {
    pub chunk: Arc<StrippedChunk>,
    pub ip: usize,
    pub return_address: Address,
    pub return_type: OperandType,
}

impl CallFrame {
    pub fn new(
        chunk: Arc<StrippedChunk>,
        return_address: Address,
        return_type: OperandType,
    ) -> Self {
        CallFrame {
            chunk,
            ip: 0,
            return_address,
            return_type,
        }
    }
}
