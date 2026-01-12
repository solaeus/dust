use std::ptr::NonNull;

use bitvec::vec::BitVec;

use crate::classes::{SizeClass, SpanClass};

pub struct Block {
    start_addr: NonNull<u8>,
    page_count: usize,

    span_class: SpanClass,
    size_class: SizeClass,
    slot_size: usize,
    slot_count: usize,

    alloc_bits: BitVec,
    free_index: usize,
    allocated_count: usize,

    heap_bits: Option<BitVec>,

    next: Option<NonNull<Block>>,
    prev: Option<NonNull<Block>>,
}

pub struct BlockList {
    head: Option<NonNull<Block>>,
    tail: Option<NonNull<Block>>,
}

impl BlockList {
    pub fn new() -> Self {
        Self {
            head: None,
            tail: None,
        }
    }
}
