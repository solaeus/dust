use std::ptr::NonNull;

use crate::{block::Block, radix_tree::RadixTree};

pub const PAGE_SIZE: usize = 8192;

pub struct PageAllocator {
    radix_tree: RadixTree,
    total_allocated: usize,
    total_freed: usize,
    release_threshold: usize,
    free_block_pool: Vec<NonNull<Block>>,
}

impl PageAllocator {
    pub fn new(release_threshold: usize) -> Self {
        Self {
            radix_tree: RadixTree::new(),
            total_allocated: 0,
            total_freed: 0,
            release_threshold,
            free_block_pool: Vec::new(),
        }
    }
}
