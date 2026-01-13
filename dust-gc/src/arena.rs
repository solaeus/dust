use memmap2::MmapMut;

pub const ARENA_SIZE: usize = 64 * 1024 * 1024;

pub struct Arena {
    base: usize,
    mmap: MmapMut,
}

#[derive(Default)]
pub struct ArenaHints {
    hints: Vec<usize>,
    next_index: usize,
}
