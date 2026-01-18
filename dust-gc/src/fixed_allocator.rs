use std::{marker::PhantomData, ptr, sync::atomic::AtomicPtr};

use crate::{
    lock_free_stack::{LockFreeNode, LockFreeStack},
    region::Region,
};

const CHUNK_SIZE: usize = 1024 * 16;

pub struct FixedAllocator<T> {
    free_list: *mut u8,

    current_chunk: *mut u8,
    curret_chunk_used: usize,

    chunks: LockFreeStack<Chunk>,

    capacity: usize,
    used: usize,

    element_size: usize,
    element_align: usize,

    _phantom: PhantomData<T>,
}

impl<T> FixedAllocator<T> {
    pub fn new() -> Self {
        let element_align = align_of::<T>().max(align_of::<*mut u8>());
        let unaligned_size = size_of::<T>().max(size_of::<*mut u8>());
        let element_size = (unaligned_size + element_align - 1) & !(element_align - 1);
        let capacity = CHUNK_SIZE / element_size;

        Self {
            free_list: ptr::null_mut(),
            current_chunk: ptr::null_mut(),
            curret_chunk_used: 0,
            chunks: LockFreeStack::new(),
            capacity,
            used: 0,
            element_size,
            element_align,
            _phantom: PhantomData,
        }
    }
}

struct Chunk {
    next: AtomicPtr<Chunk>,
    region: Region,
}

impl LockFreeNode for Chunk {
    fn link(&self) -> &AtomicPtr<Self> {
        &self.next
    }
}
