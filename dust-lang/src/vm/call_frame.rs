use crate::{JitChunk, Value, vm::Register};

#[derive(Debug)]
#[repr(C)]
pub struct CallFrame {
    pub ip: usize,
    pub chunk: *const JitChunk,
    pub is_end_of_stack: bool,
    pub registers: *mut Register,
    pub register_count: usize,
    pub return_address: usize,
    pub return_value: Option<Value>,
}
