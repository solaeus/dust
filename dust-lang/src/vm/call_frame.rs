use std::sync::Arc;

use crate::{Address, OperandType};

#[derive(Debug)]
pub struct CallFrame<C> {
    pub chunk: Arc<C>,
    pub ip: usize,
    pub return_address: Address,
    pub return_type: OperandType,
    pub skipped_registers: usize,
}

impl<C> CallFrame<C> {
    pub fn new(
        chunk: Arc<C>,
        return_address: Address,
        return_type: OperandType,
        skipped_registers: usize,
    ) -> Self {
        CallFrame {
            chunk,
            ip: 0,
            return_address,
            return_type,
            skipped_registers,
        }
    }
}
