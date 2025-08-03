pub mod offsets {
    const USIZE_SIZE: usize = size_of::<usize>();
    const PTR_SIZE: usize = size_of::<*const u8>();
    const I64_SIZE: usize = size_of::<i64>();
    const U8_SIZE: usize = size_of::<u8>();

    pub const CALL_FRAME_IP: usize = 0;
    pub const CALL_FRAME_JIT_CHUNK: usize = CALL_FRAME_IP + USIZE_SIZE;
    pub const CALL_FRAME_JIT_CHUNKS: usize = CALL_FRAME_JIT_CHUNK + PTR_SIZE;
    pub const CALL_FRAME_NEXT_CALL_INSTRUCTION: usize = CALL_FRAME_JIT_CHUNKS + PTR_SIZE;
    pub const CALL_FRAME_REGISTER_RANGE_START: usize = CALL_FRAME_NEXT_CALL_INSTRUCTION + I64_SIZE;
    pub const CALL_FRAME_REGISTER_RANGE_END: usize = CALL_FRAME_REGISTER_RANGE_START + USIZE_SIZE;
    pub const CALL_FRAME_RETURN_TYPE: usize = CALL_FRAME_REGISTER_RANGE_END + USIZE_SIZE;
    pub const CALL_FRAME_RETURN_TYPE_PAD: usize = (USIZE_SIZE - U8_SIZE) % USIZE_SIZE;
    pub const CALL_FRAME_RETURN_REGISTER_INDEX: usize =
        CALL_FRAME_RETURN_TYPE + U8_SIZE + CALL_FRAME_RETURN_TYPE_PAD;
}

use offsets::*;

pub const CALL_FRAME_SIZE: usize = CALL_FRAME_RETURN_REGISTER_INDEX + size_of::<usize>();

pub type CallFrame = [u8; CALL_FRAME_SIZE];

#[macro_export]
#[allow(clippy::crate_in_macro_def)]
macro_rules! call_frame {
    (new) => {
        let mut call_frame = vec![0u8; CALL_FRAME_SIZE];

        unsafe {
            *(call_frame.as_mut_ptr().add(CALL_FRAME_IP) as *mut usize) = 0;
            *(call_frame.as_mut_ptr().add(CALL_FRAME_JIT_CHUNK) as *mut usize) = jit_chunk;
            *(call_frame.as_mut_ptr().add(CALL_FRAME_JIT_CHUNKS) as *mut usize) = jit_chunks;
            *(call_frame
                .as_mut_ptr()
                .add(CALL_FRAME_NEXT_CALL_INSTRUCTION) as *mut i64) = 0;
            *(call_frame.as_mut_ptr().add(CALL_FRAME_REGISTER_RANGE_START) as *mut usize) =
                register_range.0;
            *(call_frame.as_mut_ptr().add(CALL_FRAME_REGISTER_RANGE_END) as *mut usize) =
                register_range.1;
            *(call_frame.as_mut_ptr().add(CALL_FRAME_RETURN_TYPE) as *mut u8) = return_type;
            *(call_frame
                .as_mut_ptr()
                .add(CALL_FRAME_RETURN_REGISTER_INDEX) as *mut usize) = return_register_index;
        };

        call_frame
    };
    (ip, $builder:expr, $frame_ptr:expr) => {
        $builder.ins().load(
            types::I64,
            MemFlags::new(),
            $frame_ptr,
            CALL_FRAME_IP as i32,
        )
    };
    (set_ip, $builder:expr, $frame_ptr:expr, $value:expr) => {
        $builder
            .ins()
            .store(MemFlags::new(), $value, $frame_ptr, CALL_FRAME_IP as i32)
    };
    (next_call_instruction, $builder:expr, $frame_ptr:expr) => {
        $builder.ins().load(
            types::I64,
            MemFlags::new(),
            $frame_ptr,
            CALL_FRAME_NEXT_CALL_INSTRUCTION as i32,
        )
    };
    (set_next_call_instruction, $builder:expr, $frame_ptr:expr, $value:expr) => {
        $builder.ins().store(
            MemFlags::new(),
            $value,
            $frame_ptr,
            CALL_FRAME_NEXT_CALL_INSTRUCTION as i32,
        )
    };
    (register_range_start, $builder:expr, $frame_ptr:expr) => {
        $builder.ins().load(
            types::I64,
            MemFlags::new(),
            $frame_ptr,
            CALL_FRAME_REGISTER_RANGE_START as i32,
        )
    };
    (register_range_end, $builder:expr, $frame_ptr:expr) => {
        $builder.ins().load(
            types::I64,
            MemFlags::new(),
            $frame_ptr,
            CALL_FRAME_REGISTER_RANGE_END as i32,
        )
    };
    (return_register_index, $builder:expr, $frame_ptr:expr) => {
        $builder.ins().load(
            types::I64,
            MemFlags::new(),
            $frame_ptr,
            CALL_FRAME_RETURN_REGISTER_INDEX as i32,
        )
    };
    (set_return_register_index, $builder:expr, $frame_ptr:expr, $value:expr) => {
        $builder.ins().store(
            MemFlags::new(),
            $value,
            $frame_ptr,
            CALL_FRAME_RETURN_REGISTER_INDEX as i32,
        )
    };
}
