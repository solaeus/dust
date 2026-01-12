use memmap2::MmapMut;

use crate::ARENA_SIZE;

pub struct Arena {
    base: usize,
    mmap: MmapMut,
}

impl Arena {
    pub fn base(&self) -> usize {
        self.base
    }

    pub fn size(&self) -> usize {
        ARENA_SIZE
    }

    pub fn contains(&self, addr: usize) -> bool {
        todo!()
    }
}

pub struct ArenaHints {
    hints: Vec<usize>,
    next_index: usize,
}

impl ArenaHints {
    pub fn next(&mut self) -> Option<usize> {
        todo!()
    }

    pub fn update_for_arena(&mut self, arena: &Arena) {
        todo!()
    }
}
