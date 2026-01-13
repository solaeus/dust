use std::{array, sync::Mutex};

use crate::{
    arena::{Arena, ArenaHints},
    block::SizeClass,
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
}
