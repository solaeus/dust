const RADIX_TREE_HEIGHT: usize = 5;
const RADIX_TREE_FANOUT: usize = 8;
const SUMMARY_MAX_PAGES: usize = 1 << 21;

#[derive(Clone, Copy)]
pub struct PallocSum(u64);

impl PallocSum {
    pub fn new(start: usize, end: usize, max: usize) -> Self {
        todo!()
    }

    pub fn start(&self) -> usize {
        todo!()
    }

    pub fn end(&self) -> usize {
        todo!()
    }

    pub fn max(&self) -> usize {
        todo!()
    }

    pub fn merge(s1: Self, s2: Self) -> Self {
        todo!()
    }

    pub fn is_all_free(&self) -> bool {
        todo!()
    }

    pub fn is_all_used(&self) -> bool {
        self.0 == 0
    }
}

pub struct RadixTree {
    levels: [Vec<u64>; RADIX_TREE_HEIGHT],
    search_addr: u64,
}

impl RadixTree {
    pub fn new() -> Self {
        todo!()
    }

    pub fn find_free_pages(&self, npages: usize) -> Option<usize> {
        todo!()
    }

    pub fn alloc_range(&self, base: usize, npages: usize) {
        todo!()
    }

    pub fn free_range(&self, base: usize, npages: usize) {
        todo!()
    }

    pub fn grow(&mut self, base: usize, npages: usize) {
        todo!()
    }

    fn update_summaries(&self, page_index: usize) {
        todo!()
    }

    fn get_summary(&self, level: usize, index: usize) -> PallocSum {
        todo!()
    }

    fn set_summary(&self, level: usize, index: usize, sum: PallocSum) {
        todo!()
    }

    fn search_addr(&self) -> usize {
        todo!()
    }

    fn set_search_addr(&self, addr: usize) {
        todo!()
    }
}

pub struct PageBitmap {
    bits: [u64; 8],
}

impl PageBitmap {
    pub fn new() -> Self {
        todo!()
    }

    pub fn alloc(&mut self, n: usize) -> Option<usize> {
        todo!()
    }

    pub fn free(&mut self, index: usize) {
        todo!()
    }

    pub fn summary(&self) -> PallocSum {
        todo!()
    }
}
