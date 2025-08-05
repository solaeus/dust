pub mod sizes {
    pub const IP: usize = 0;
    pub const FUNCTION_INDEX: usize = IP + size_of::<usize>();
    pub const CALL_INSTRUCTION: usize = FUNCTION_INDEX + size_of::<usize>();
    pub const REGISTER_RANGE_START: usize = CALL_INSTRUCTION + size_of::<usize>();
    pub const REGISTER_RANGE_END: usize = REGISTER_RANGE_START + size_of::<usize>();

    pub const CALL_FRAME_SIZE: usize = REGISTER_RANGE_END + size_of::<usize>();
}

macro_rules! call_stack {
    (new, $capacity: expr) => {{
        pub use crate::jit_vm::call_stack::sizes::*;

        let mut call_stack = Vec::with_capacity($capacity);

        call_stack.resize($capacity * CALL_FRAME_SIZE, 0);

        call_stack
    }};
    (ip_offset, $frame_index: expr, $call_stack: expr) => {{
        pub use crate::jit_vm::call_stack::sizes::*;

        $frame_index * CALL_FRAME_SIZE + IP
    }};
    (function_index_offset, $frame_index: expr, $call_stack: expr) => {{
        pub use crate::jit_vm::call_stack::sizes::*;

        $frame_index * CALL_FRAME_SIZE + FUNCTION_INDEX
    }};
    (call_instruction_offset, $frame_index: expr, $call_stack: expr) => {{
        pub use crate::jit_vm::call_stack::sizes::*;

        $frame_index * CALL_FRAME_SIZE + CALL_INSTRUCTION
    }};
    (register_range_start_offset, $frame_index: expr, $call_stack: expr) => {{
        pub use crate::jit_vm::call_stack::sizes::*;

        $frame_index * CALL_FRAME_SIZE + REGISTER_RANGE_START
    }};
    (register_range_end_offset, $frame_index: expr, $call_stack: expr) => {{
        pub use crate::jit_vm::call_stack::sizes::*;

        $frame_index * CALL_FRAME_SIZE + REGISTER_RANGE_END
    }};
}
