use crate::block::Block;
use crate::classes::SpanClass;
use crate::radix_tree::RadixTree;
use std::alloc::AllocError;
use std::ptr::NonNull;

pub struct PageAllocator {
    radix_tree: RadixTree,
    total_allocated: usize,
    total_freed: usize,
    release_threshold: usize,
    free_block_pool: Vec<NonNull<Block>>,
}

impl PageAllocator {
    pub fn new() -> Self {
        todo!()
    }

    pub fn alloc_block(
        &mut self,
        span_class: SpanClass,
        npages: usize,
    ) -> Result<NonNull<Block>, AllocError> {
        todo!()
    }

    pub fn free_block(&mut self, block: NonNull<Block>) {
        todo!()
    }

    pub fn grow(&mut self, base: usize, npages: usize) {
        todo!()
    }

    fn alloc_block_metadata(&mut self) -> Result<NonNull<Block>, AllocError> {
        todo!()
    }

    fn free_block_metadata(&mut self, block: NonNull<Block>) {
        todo!()
    }

    fn find_free_pages(&self, npages: usize) -> Option<usize> {
        todo!()
    }

    fn alloc_pages(&mut self, base: usize, npages: usize) {
        todo!()
    }

    fn free_pages(&mut self, base: usize, npages: usize) {
        todo!()
    }

    fn setup_block(
        &mut self,
        base: usize,
        npages: usize,
        span_class: SpanClass,
    ) -> Result<NonNull<Block>, AllocError> {
        todo!()
    }

    fn should_release_memory(&self) -> bool {
        todo!()
    }

    fn release_empty_blocks(&mut self) {
        todo!()
    }

    pub fn total_allocated(&self) -> usize {
        self.total_allocated
    }

    pub fn total_freed(&self) -> usize {
        self.total_freed
    }
}
