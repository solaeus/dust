use crate::{page_allocator::PAGE_SIZE, region::Region};

pub const PAGES_PER_ARENA: usize = 64;
pub const ARENA_SIZE: usize = PAGES_PER_ARENA * PAGE_SIZE;
const ARENA_BASE_OFFSET: usize = if cfg!(all(target_vendor = "amd", target_pointer_width = "64")) {
    0xffff800000000000
} else if cfg!(any(target_os = "aix", target_arch = "powerpc64")) {
    0x0a00000000000000
} else {
    0
};

pub struct Arena {
    region: Region,
    pub free_pages: u64,
    pub scavenged_pages: u64,
}

impl Arena {
    pub fn new() -> Self {
        Self {
            region: Region::new(ARENA_SIZE).expect("ARENA_SIZE must not be zero"),
            free_pages: 0,
            scavenged_pages: 0,
        }
    }

    pub fn base(&self) -> usize {
        self.region.base()
    }
}
