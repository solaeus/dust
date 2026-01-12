use crate::GlobalHeap;
use crate::block::Block;
use crate::block_cache::BlockCache;
use crate::classes::{SizeClass, SpanClass};
use crate::page_cache::PageCache;
use std::alloc::AllocError;
use std::ptr::NonNull;
use std::sync::Arc;

const TINY_SLOT_SIZE: usize = 16;
const TINY_MAX_SIZE: usize = 16;

pub struct ThreadHeap {
    parent: Arc<GlobalHeap>,

    blocks: [Option<NonNull<Block>>; SpanClass::SPAN_CLASS_COUNT],

    tiny: Option<NonNull<u8>>,
    tiny_offset: usize,
    tiny_count: usize,

    page_cache: PageCache,

    free_block_cache: Option<NonNull<Block>>,
}

impl ThreadHeap {
    pub fn new() -> Self {
        todo!()
    }

    pub fn alloc_tiny(
        &mut self,
        size: usize,
        noscan: bool,
        caches: &[BlockCache],
    ) -> Result<NonNull<u8>, AllocError> {
        todo!()
    }

    pub fn alloc_small(
        &mut self,
        size: usize,
        noscan: bool,
        caches: &[BlockCache],
    ) -> Result<NonNull<u8>, AllocError> {
        todo!()
    }

    pub fn alloc_large(&mut self, size: usize, noscan: bool) -> Result<NonNull<u8>, AllocError> {
        todo!()
    }

    fn alloc_from_block(
        &mut self,
        span_class: SpanClass,
        caches: &[BlockCache],
    ) -> Result<NonNull<u8>, AllocError> {
        todo!()
    }

    fn refill_block(
        &mut self,
        span_class: SpanClass,
        caches: &[BlockCache],
    ) -> Result<(), AllocError> {
        todo!()
    }

    fn return_block(&mut self, span_class: SpanClass, caches: &[BlockCache]) {
        todo!()
    }

    pub fn page_cache_mut(&mut self) -> &mut PageCache {
        &mut self.page_cache
    }

    pub fn flush(&mut self, caches: &[BlockCache]) {
        todo!()
    }
}
