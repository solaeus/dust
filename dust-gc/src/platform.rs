use crate::page_allocator::PAGE_SIZE;

#[cfg(target_pointer_width = "64")]
pub use pointer_width_64::*;

#[cfg(target_pointer_width = "32")]
pub use pointer_width_32::*;

mod pointer_width_64 {
    use super::*;

    pub const HEAP_ADDRESS_BITS: usize = 48;
    pub const INFO_LEVELS: usize = 5;
    pub const PAGES_PER_SPAN: usize = 512;
    pub const BYTES_PER_SPAN: usize = PAGE_SIZE * PAGES_PER_SPAN;

    pub const LEVEL_0_BITS: usize = 14;
    pub const LEVEL_BITS: usize = 3;

    pub const INFO_LEVEL_BITS: [usize; INFO_LEVELS] =
        [LEVEL_0_BITS, LEVEL_BITS, LEVEL_BITS, LEVEL_BITS, LEVEL_BITS];
    pub const INFO_LEVEL_SHIFTS: [usize; INFO_LEVELS] = [
        HEAP_ADDRESS_BITS - LEVEL_0_BITS,
        HEAP_ADDRESS_BITS - LEVEL_0_BITS - LEVEL_BITS,
        HEAP_ADDRESS_BITS - LEVEL_0_BITS - 2 * LEVEL_BITS,
        HEAP_ADDRESS_BITS - LEVEL_0_BITS - 3 * LEVEL_BITS,
        HEAP_ADDRESS_BITS - LEVEL_0_BITS - 4 * LEVEL_BITS,
    ];

    pub const MAX_PACKED_VALUE: u32 = 1024 * 1024 * 2;
}

mod pointer_width_32 {
    use super::*;

    pub const HEAP_ADDRESS_BITS: usize = 32;
    pub const INFO_LEVELS: usize = 4;
    pub const PAGES_PER_CHUNK: usize = 64;
    pub const BYTES_PER_CHUNK: usize = PAGE_SIZE * PAGES_PER_CHUNK;

    pub const LEVEL_0_BITS: usize = 4;
    pub const LEVEL_BITS: usize = 3;

    pub const INFO_LEVEL_BITS: [usize; INFO_LEVELS] =
        [LEVEL_0_BITS, LEVEL_BITS, LEVEL_BITS, LEVEL_BITS];
    pub const INFO_LEVEL_SHIFTS: [usize; INFO_LEVELS] = [
        HEAP_ADDRESS_BITS - LEVEL_0_BITS,
        HEAP_ADDRESS_BITS - LEVEL_0_BITS - LEVEL_BITS,
        HEAP_ADDRESS_BITS - LEVEL_0_BITS - 2 * LEVEL_BITS,
        HEAP_ADDRESS_BITS - LEVEL_0_BITS - 3 * LEVEL_BITS,
    ];

    pub const MAX_PACKED_VALUE: u32 = 1024 * 32;
}
