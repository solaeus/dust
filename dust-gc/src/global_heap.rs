use std::{array, ptr::NonNull, sync::Mutex};

use crate::{
    arena::{Arena, ArenaHints},
    block::{Block, SizeClass},
    block_cache::BlockCache,
    page_allocator::PageAllocator,
};

pub struct GlobalHeap {
    page_allocator: Mutex<PageAllocator>,
    block_caches: [Mutex<BlockCache>; SizeClass::CLASS_COUNT],
    arenas: Vec<Arena>,
    arena_hints: ArenaHints,
}

impl GlobalHeap {
    pub fn new(memory_free_threshold: usize) -> Self {
        Self {
            page_allocator: Mutex::new(PageAllocator::new(memory_free_threshold)),
            block_caches: array::from_fn(|index| {
                Mutex::new(BlockCache::new(SizeClass::from_index(index)))
            }),
            arenas: Vec::new(),
            arena_hints: ArenaHints::default(),
        }
    }

    pub(crate) fn allocate_block(&self, size_class: SizeClass) -> Option<NonNull<Block>> {
        let class_index = size_class.index();

        assert!(class_index < SizeClass::CLASS_COUNT);

        let mut cache = self.block_caches[class_index]
            .lock()
            .expect("Failed to lock block cache");

        if let Some(block_pointer) = cache.pop_partial() {
            return Some(block_pointer);
        }

        drop(cache);

        let mut page_allocator = self
            .page_allocator
            .lock()
            .expect("Failed to lock page allocator");
        let block = page_allocator.allocate_block(size_class)?;

        todo!()
    }
}
