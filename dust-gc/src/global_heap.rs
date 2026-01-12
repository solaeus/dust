use std::{alloc::AllocError, ptr::NonNull, sync::Mutex};

use crate::{
    arena::{Arena, ArenaHints},
    block::Block,
    block_cache::BlockCache,
    classes::SpanClass,
    page_allocator::PageAllocator,
    page_cache::PageCache,
    thread_heap::ThreadHeap,
};

pub struct GlobalHeap {
    page_allocator: Mutex<PageAllocator>,
    block_caches: [Mutex<BlockCache>; SpanClass::SPAN_CLASS_COUNT],
    arenas: Vec<Arena>,
    arena_hints: ArenaHints,
}

impl GlobalHeap {
    pub fn new() -> Self {
        todo!()
    }

    pub fn alloc(
        &self,
        thread_heap: &mut ThreadHeap,
        size: usize,
        noscan: bool,
    ) -> Result<NonNull<u8>, AllocError> {
        todo!()
    }

    pub fn alloc_tiny(
        &self,
        thread_heap: &mut ThreadHeap,
        size: usize,
        noscan: bool,
    ) -> Result<NonNull<u8>, AllocError> {
        todo!()
    }

    pub fn alloc_small(
        &self,
        thread_heap: &mut ThreadHeap,
        size: usize,
        noscan: bool,
    ) -> Result<NonNull<u8>, AllocError> {
        todo!()
    }

    pub fn alloc_large(
        &self,
        thread_heap: &mut ThreadHeap,
        size: usize,
        noscan: bool,
    ) -> Result<NonNull<u8>, AllocError> {
        todo!()
    }

    pub fn alloc_block(
        &self,
        span_class: SpanClass,
        npages: usize,
    ) -> Result<NonNull<Block>, AllocError> {
        todo!()
    }

    pub fn alloc_block_from_cache(
        &self,
        page_cache: &mut PageCache,
        span_class: SpanClass,
        npages: usize,
    ) -> Result<Option<NonNull<Block>>, AllocError> {
        todo!()
    }

    pub fn free_block(&self, block: NonNull<Block>) {
        todo!()
    }

    pub fn grow(&mut self, npages: usize) -> Result<usize, AllocError> {
        todo!()
    }

    pub fn total_allocated(&self) -> usize {
        todo!()
    }

    pub fn total_freed(&self) -> usize {
        todo!()
    }
}
