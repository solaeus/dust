use std::sync::Arc;

use crate::{Address, OperandType, panic_vm::memory::Register};

#[derive(Debug)]
pub struct CallFrame<'a, C> {
    pub chunk: Arc<C>,
    pub ip: usize,
    pub return_address: Address,
    pub return_type: OperandType,
    pub registers: &'a mut [Register],
}

impl<'a, C> CallFrame<'a, C> {
    pub fn new(
        chunk: Arc<C>,
        return_address: Address,
        return_type: OperandType,
        registers: &'a mut [Register],
    ) -> Self {
        CallFrame {
            chunk,
            ip: 0,
            return_address,
            return_type,
            registers,
        }
    }
}
