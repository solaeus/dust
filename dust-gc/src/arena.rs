use std::{
    alloc::{Layout, alloc_zeroed, handle_alloc_error},
    mem::forget,
    ptr::NonNull,
};

use crate::page_allocator::PAGE_SIZE;

const PAGES_PER_ARENA: usize = 64;
const ARENA_SIZE: usize = PAGES_PER_ARENA * PAGE_SIZE;

pub struct Arena {
    base: usize,
    free_pages: u64,
    scavenged_pages: u64,
}

impl Arena {
    pub fn new() -> Self {
        if cfg!(target_arch = "wasm32") {
            Self::new_from_global_allocator()
        } else {
            Self::new_from_mmap()
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn new_from_mmap() -> Self {
        use memmap2::MmapOptions;

        let mut mmap = MmapOptions::new()
            .len(ARENA_SIZE)
            .map_anon()
            .expect("Tried to create memory arena that is too large for this platform.");
        let base = mmap.as_mut_ptr().addr();

        forget(mmap);

        Self {
            base,
            free_pages: 0,
            scavenged_pages: 0,
        }
    }

    fn new_from_global_allocator() -> Self {
        let layout = Layout::from_size_align(ARENA_SIZE, 1)
            .expect("Tried to create memory arena that is too large for this platform.");
        let raw_pointer = unsafe { alloc_zeroed(layout) };

        if raw_pointer.is_null() {
            handle_alloc_error(layout);
        }

        let base = raw_pointer.addr();

        Arena {
            base,
            free_pages: 0,
            scavenged_pages: 0,
        }
    }
}
