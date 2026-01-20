use std::{
    ptr::NonNull,
    sync::atomic::{AtomicPtr, AtomicU32, AtomicU64, AtomicUsize},
};

use parking_lot::Mutex;

use crate::{
    lock_free_stack::{LockFreeNode, LockFreeStack},
    span::Span,
};

const BLOCK_ENTRIES: usize = 512;
const SPAN_SET_STARTING_SPINE_CAPACITY: usize = 256;

pub struct SpanSet {
    spine: AtomicPtr<AtomicPtr<Block>>,
    spine_capacity: Mutex<usize>,
    spine_length: AtomicUsize,
    head_tail_index: AtomicU64,
}

impl SpanSet {
    pub fn new() -> Self {
        Self {
            spine: AtomicPtr::null(),
            spine_capacity: Mutex::new(0),
            spine_length: AtomicUsize::new(0),
            head_tail_index: AtomicU64::new(0),
        }
    }

    pub fn push(&self, span: NonNull<Span>) {
        todo!()
    }

    pub fn pop(&self) -> Option<NonNull<Span>> {
        todo!()
    }
}

unsafe impl Send for SpanSet {}
unsafe impl Sync for SpanSet {}

struct BlockAllocator {
    stack: LockFreeStack<Block>,
}

impl BlockAllocator {
    fn allocate(&self) -> Option<NonNull<Block>> {
        self.stack.pop()
    }
}

struct Block {
    popped: AtomicU32,
    spans: [AtomicPtr<Span>; BLOCK_ENTRIES],
    free_list_link: AtomicPtr<Self>,
}

impl LockFreeNode for Block {
    fn link(&self) -> &AtomicPtr<Self> {
        &self.free_list_link
    }
}
