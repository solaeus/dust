use crate::{Instruction, JitChunk, OperandType, Register};

#[repr(C)]
pub struct CallFrame<'a> {
    pub ip: usize,
    pub jit_chunk: &'a JitChunk,
    pub jit_chunks: &'a [JitChunk],
    pub next_call: Instruction,
    pub register_range: (usize, usize),
    pub return_address_index: usize,
    pub return_type: OperandType,
    pub return_register: Register,
}

impl<'a> CallFrame<'a> {
    pub fn new(
        jit_chunk: &'a JitChunk,
        jit_chunks: &'a [JitChunk],
        register_range: (usize, usize),
        return_address_index: usize,
        return_type: OperandType,
    ) -> Self {
        CallFrame {
            jit_chunk,
            jit_chunks,
            register_range,
            return_address_index,
            return_type,
            ip: 0,
            next_call: Instruction::no_op(),
            return_register: Register { empty: () },
        }
    }
}
