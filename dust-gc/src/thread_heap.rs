use std::{ptr::NonNull, sync::Arc};

use crate::{
    GlobalHeap,
    block::{Block, SizeClass},
};

const TINY_SLOT_SIZE: usize = 16;

pub struct ThreadHeap {
    parent: Arc<GlobalHeap>,

    blocks: [Option<NonNull<Block>>; SizeClass::CLASS_COUNT],

    tiny: Option<NonNull<u8>>,
    tiny_offset: usize,
    tiny_count: usize,

    free_block_cache: Option<NonNull<Block>>,
}

impl ThreadHeap {
    pub fn new(parent: Arc<GlobalHeap>) -> Self {
        Self {
            parent,
            blocks: [None; SizeClass::CLASS_COUNT],
            tiny: None,
            tiny_offset: 0,
            tiny_count: 0,
            free_block_cache: None,
        }
    }

    pub fn allocate(&mut self, size: usize) -> NonNull<u8> {
        let size_class = SizeClass::from_size(size);

        if size_class.is_tiny() {
            return self.allocate_tiny(size);
        }

        if size_class.is_large() {
            return self.allocate_large(size);
        }

        let existing_block = self.blocks[size_class.index()];

        todo!()
    }

    fn allocate_tiny(&mut self, size: usize) -> NonNull<u8> {
        todo!()
    }

    fn allocate_large(&self, size: usize) -> NonNull<u8> {
        todo!()
    }
}
