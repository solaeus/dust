use std::ptr::NonNull;

use crate::block::{Block, SizeClass};

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

    pub fn allocate_block(&mut self, size_class: SizeClass) -> Option<NonNull<Block>> {
        let page_count = size_class.page_count();

        debug_assert_ne!(page_count, 0);

        let first_page = self.allocate_pages(page_count)?;
        let base_address = first_page * PAGE_SIZE;
        let start_pointer = unsafe { NonNull::new_unchecked(base_address as *mut u8) };

        if let Some(block_pointer) = self.free_block_pool.pop() {
            let block = unsafe { &mut *block_pointer.as_ptr() };

            block.reuse(start_pointer, page_count, size_class);

            Some(block_pointer)
        } else {
            Some(Block::new(start_pointer, page_count, size_class))
        }
    }

    fn allocate_pages(&mut self, page_count: usize) -> Option<usize> {
        let page_index = self.radix_tree.find_free(page_count)?;

        self.total_allocated += page_count * PAGE_SIZE;

        Some(page_index)
    }
}

pub struct RadixTree {
    levels: [[PageInfo; Self::PAGE_COUNT_PER_LEVEL]; Self::LEVEL_COUNT],
    search_page: usize,
}

impl RadixTree {
    const LEVEL_COUNT: usize = 5;
    const FANOUT: usize = 8;
    const PAGE_COUNT_PER_LEVEL: usize = 512;
    const TOTAL_PAGES: usize = {
        let mut total = 0;
        let mut level = 0;

        while level < Self::LEVEL_COUNT {
            total += Self::PAGE_COUNT_PER_LEVEL * Self::FANOUT.pow(level as u32);
            level += 1;
        }

        total
    };

    pub const fn new() -> Self {
        let empty_leaf = PageInfo::new(
            Self::PAGE_COUNT_PER_LEVEL,
            Self::PAGE_COUNT_PER_LEVEL,
            Self::PAGE_COUNT_PER_LEVEL,
        );

        Self {
            levels: [[empty_leaf; Self::PAGE_COUNT_PER_LEVEL]; Self::LEVEL_COUNT],
            search_page: 0,
        }
    }

    pub fn find_free(&self, page_count: usize) -> Option<usize> {
        assert_ne!(page_count, 0);
        assert!(page_count <= Self::PAGE_COUNT_PER_LEVEL);

        let mut page_index = self.search_page / Self::PAGE_COUNT_PER_LEVEL % Self::LEVEL_COUNT;
        let start_index = page_index;
        let mut wrapped = false;

        while !wrapped || page_index != start_index {
            let info = self.get_info(0, page_index);

            if info.max() >= page_count {
                let base_index = page_index * Self::PAGE_COUNT_PER_LEVEL;

                return Some(base_index);
            }

            page_index += 1;

            if page_index >= Self::PAGE_COUNT_PER_LEVEL {
                page_index = 0;

                if wrapped {
                    break;
                }

                wrapped = true;
            }
        }

        None
    }

    pub fn update(&mut self, leaf_index: usize, leaf_bitmap: &PageBitmap) {
        assert!(leaf_index < Self::PAGE_COUNT_PER_LEVEL);

        let leaf_info = leaf_bitmap.info();

        self.set_info(0, leaf_index, leaf_info);

        for level in 1..Self::LEVEL_COUNT {
            let parent_index = leaf_index / Self::FANOUT;
            let left_child_index = parent_index * Self::FANOUT;
            let right_child_index = left_child_index + Self::FANOUT - 1;

            let left_info = self.get_info(level - 1, left_child_index);
            let right_info = self.get_info(level - 1, right_child_index);
            let parent_info = left_info.merge(right_info, level);

            self.set_info(level, parent_index, parent_info);
        }
    }

    fn get_info(&self, level: usize, leaf: usize) -> PageInfo {
        assert!(level < Self::LEVEL_COUNT);
        assert!(leaf < Self::PAGE_COUNT_PER_LEVEL);

        self.levels[level][leaf]
    }

    fn set_info(&mut self, level: usize, leaf: usize, info: PageInfo) {
        assert!(level < Self::LEVEL_COUNT);
        assert!(leaf < Self::PAGE_COUNT_PER_LEVEL);

        self.levels[level][leaf] = info;
    }

    const fn page_count_for_level(level: usize) -> usize {
        Self::PAGE_COUNT_PER_LEVEL * Self::FANOUT.pow(level as u32)
    }
}

#[derive(Clone, Copy)]
pub struct PageInfo(u64);

impl PageInfo {
    const FIELD_BITS: usize = 21;
    const FIELD_MASK: u64 = (1 << Self::FIELD_BITS) - 1;

    pub const fn new(start: usize, end: usize, max: usize) -> Self {
        let packed = (max << (Self::FIELD_BITS * 2)) | (end << Self::FIELD_BITS) | start;

        PageInfo(packed as u64)
    }

    pub fn start(&self) -> usize {
        (self.0 & Self::FIELD_MASK) as usize
    }

    pub fn end(&self) -> usize {
        ((self.0 >> Self::FIELD_BITS) & Self::FIELD_MASK) as usize
    }

    pub fn max(&self) -> usize {
        ((self.0 >> (Self::FIELD_BITS * 2)) & Self::FIELD_MASK) as usize
    }

    pub fn merge(self, other: Self, level: usize) -> Self {
        assert_ne!(level, 0, "Cannot merge below level 0");

        let child_level = level - 1;

        let self_start = self.start();
        let self_end = self.end();
        let self_max = self.max();

        let other_start = other.start();
        let other_end = other.end();
        let other_max = other.max();

        let start = if self_start == child_level {
            child_level + other_start
        } else {
            self_start
        };
        let end = if other_end == child_level {
            child_level + self_end
        } else {
            other_end
        };
        let max = self_max.max(other_max).max(self_end + other_start);

        PageInfo::new(start, end, max)
    }

    pub fn is_empty(&self, span: usize) -> bool {
        let start = self.start();

        start == span && start == self.end() && start == self.max()
    }

    pub fn is_full(&self) -> bool {
        self.0 == 0
    }
}

#[derive(Default)]
pub struct PageBitmap {
    bits: [u64; RadixTree::PAGE_COUNT_PER_LEVEL / 64],
}

impl PageBitmap {
    pub fn allocate(&mut self, n: usize) -> Option<usize> {
        if n == 0 || n > 512 {
            return None;
        }

        let mut run_start = None;
        let mut run_length = 0;

        for index in 0..RadixTree::PAGE_COUNT_PER_LEVEL {
            let word = index / 64;
            let bit = index % 64;
            let used = (self.bits[word] >> bit) & 1 == 1;

            if used {
                run_start = None;
                run_length = 0;
            } else {
                if run_start.is_none() {
                    run_start = Some(index);
                }

                run_length += 1;

                if run_length == n {
                    let start = run_start.unwrap();

                    for i in start..start + n {
                        let w = i / 64;
                        let b = i % 64;
                        self.bits[w] |= 1 << b;
                    }

                    return Some(start);
                }
            }
        }

        None
    }

    pub fn free(&mut self, index: usize) {
        if index >= RadixTree::PAGE_COUNT_PER_LEVEL {
            return;
        }

        let word = index / 64;
        let bit = index % 64;
        self.bits[word] &= !(1 << bit);
    }

    pub fn info(&self) -> PageInfo {
        let mut start = 0;
        let mut end = 0;
        let mut max = 0;

        for index in 0..RadixTree::PAGE_COUNT_PER_LEVEL {
            let word = index / 64;
            let bit = index % 64;
            let used = (self.bits[word] >> bit) & 1 == 1;

            if used {
                break;
            }

            start += 1;
        }

        for index in (0..RadixTree::PAGE_COUNT_PER_LEVEL).rev() {
            let word = index / 64;
            let bit = index % 64;
            let used = (self.bits[word] >> bit) & 1 == 1;

            if used {
                break;
            }

            end += 1;
        }

        let mut current_run = 0;

        for index in 0..RadixTree::PAGE_COUNT_PER_LEVEL {
            let word = index / 64;
            let bit = index % 64;
            let used = (self.bits[word] >> bit) & 1 == 1;

            if used {
                if current_run > max {
                    max = current_run;
                }
                current_run = 0;
            } else {
                current_run += 1;
            }
        }

        PageInfo::new(start, end, max)
    }
}
