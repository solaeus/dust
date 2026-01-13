use std::{ptr::NonNull, sync::Arc};

use crate::{
    GlobalHeap,
    block::{Block, SizeClass},
    page_cache::PageCache,
};

const TINY_SLOT_SIZE: usize = 16;

pub struct ThreadHeap {
    parent: Arc<GlobalHeap>,

    blocks: [Option<NonNull<Block>>; SizeClass::CLASS_COUNT],

    tiny: Option<NonNull<u8>>,
    tiny_offset: usize,
    tiny_count: usize,

    page_cache: PageCache,

    free_block_cache: Option<NonNull<Block>>,
}
