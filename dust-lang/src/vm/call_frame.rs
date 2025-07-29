use std::sync::Arc;

use crate::{Address, Chunk, OperandType, Value, vm::Register};

#[repr(C)]
pub struct CallFrame<'a> {
    pub ip: usize,
    pub chunk: Arc<Chunk>,
    pub is_end_of_stack: bool,
    pub registers: &'a mut [Register],
    pub return_address: Address,
    pub return_type: OperandType,
    pub return_value: Option<Option<Value>>,
}

impl<'a> CallFrame<'a> {
    pub fn new(
        chunk: Arc<Chunk>,
        registers: &'a mut [Register],
        is_end_of_stack: bool,
        return_address: Address,
        return_type: OperandType,
    ) -> Self {
        CallFrame {
            ip: 0,
            chunk,
            registers,
            is_end_of_stack,
            return_address,
            return_type,
            return_value: None,
        }
    }
}
