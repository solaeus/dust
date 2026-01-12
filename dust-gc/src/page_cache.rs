use std::sync::atomic::{AtomicU64, Ordering};

const PAGE_CACHE_SIZE: usize = 64;

pub struct PageCache {
    base: usize,
    cache: AtomicU64,
}

impl PageCache {
    pub fn new() -> Self {
        todo!()
    }

    pub fn alloc(&mut self, npages: usize) -> Option<usize> {
        todo!()
    }

    pub fn flush(&mut self) -> Option<(usize, u64)> {
        todo!()
    }

    pub fn empty(&self) -> bool {
        todo!()
    }
}
