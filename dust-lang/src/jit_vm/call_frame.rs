use crate::{JitChunk, OperandType};

#[repr(C)]
pub struct CallFrame<'a> {
    pub ip: usize,
    pub jit_chunk: &'a JitChunk,
    pub jit_chunks: &'a [JitChunk],
    pub next_call_instruction: i64,
    pub register_range: (usize, usize),
    pub return_type: OperandType,
    pub return_register_index: usize,
}

impl<'a> CallFrame<'a> {
    pub fn new(
        jit_chunk: &'a JitChunk,
        jit_chunks: &'a [JitChunk],
        register_range: (usize, usize),
        return_type: OperandType,
        return_register_index: usize,
    ) -> Self {
        CallFrame {
            ip: 0,
            next_call_instruction: 0,
            jit_chunk,
            jit_chunks,
            register_range,
            return_type,
            return_register_index,
        }
    }
}
