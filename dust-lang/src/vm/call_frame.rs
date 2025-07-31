use crate::{JitChunk, OperandType, Register};

#[repr(C)]
pub struct CallFrame {
    pub register_range: (usize, usize),

    pub ip: usize,
    pub jit_chunk: JitChunk,
    pub push_back: bool,

    pub is_end_of_stack: bool,
    pub return_address_index: usize,
    pub return_type: OperandType,
    pub return_register: Register,
}

impl CallFrame {
    pub fn new(
        jit_chunk: JitChunk,
        register_range: (usize, usize),
        is_end_of_stack: bool,
        return_address_index: usize,
        return_type: OperandType,
    ) -> Self {
        CallFrame {
            ip: 0,
            jit_chunk,
            push_back: false,
            register_range,
            is_end_of_stack,
            return_address_index,
            return_type,
            return_register: Register { empty: () },
        }
    }
}
