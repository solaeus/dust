use std::ptr::NonNull;

use bitvec::vec::BitVec;

use crate::page_allocator::PAGE_SIZE;

pub struct Block {
    start_addr: NonNull<u8>,
    page_count: usize,

    size_class: SizeClass,
    slot_size: usize,
    slot_count: usize,

    alloc_bits: BitVec,
    free_index: usize,
    allocated_count: usize,

    heap_bits: Option<BitVec>,

    next: Option<NonNull<Block>>,
    prev: Option<NonNull<Block>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SizeClass(u8);

impl SizeClass {
    pub const CLASS_COUNT: usize = 68;
    pub const LARGE: SizeClass = SizeClass(0);

    const SIZES: [usize; Self::CLASS_COUNT] = [
        0, 8, 16, 24, 32, 48, 64, 80, 96, 112, 128, 144, 160, 176, 192, 208, 224, 240, 256, 288,
        320, 352, 384, 416, 448, 480, 512, 576, 640, 704, 768, 896, 1024, 1152, 1280, 1408, 1536,
        1792, 2048, 2304, 2688, 3072, 3200, 3456, 4096, 4864, 5376, 6144, 6528, 6784, 6912, 8192,
        9472, 9728, 10240, 10880, 12288, 13568, 14336, 16384, 18432, 19072, 20480, 21760, 24576,
        27264, 28672, 32768,
    ];

    pub fn from_index(index: usize) -> Self {
        SizeClass(index.min(Self::CLASS_COUNT - 1) as u8)
    }

    pub fn from_size(size: usize) -> Self {
        for (index, class_size) in Self::SIZES.iter().enumerate().skip(1) {
            if size <= *class_size {
                return SizeClass(index as u8);
            }
        }

        Self::LARGE
    }

    pub const fn size(&self) -> usize {
        Self::SIZES[self.0 as usize]
    }

    pub const fn index(&self) -> u8 {
        self.0
    }

    pub const fn page_count(&self) -> usize {
        self.size().div_ceil(PAGE_SIZE)
    }

    pub fn is_large(&self) -> bool {
        self == &Self::LARGE
    }
}

pub struct BlockList {
    head: Option<NonNull<Block>>,
    tail: Option<NonNull<Block>>,
}

impl BlockList {
    pub fn new() -> Self {
        Self {
            head: None,
            tail: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn size_class_from_size() {
        assert_eq!(SizeClass::from_size(0), SizeClass(1));
        assert_eq!(SizeClass::from_size(1), SizeClass(1));
        assert_eq!(SizeClass::from_size(8), SizeClass(1));
        assert_eq!(SizeClass::from_size(9), SizeClass(2));
        assert_eq!(SizeClass::from_size(15), SizeClass(2));
        assert_eq!(SizeClass::from_size(16), SizeClass(2));
        assert_eq!(SizeClass::from_size(17), SizeClass(3));
        assert_eq!(SizeClass::from_size(1000), SizeClass(32));
        assert_eq!(SizeClass::from_size(4096), SizeClass(44));
        assert_eq!(SizeClass::from_size(32768), SizeClass(67));
        assert_eq!(SizeClass::from_size(32769), SizeClass::LARGE);
    }

    #[test]
    fn size_clsss_page_count() {
        assert_eq!(SizeClass(1).page_count(), 1);
        assert_eq!(SizeClass(44).page_count(), 1);
        assert_eq!(SizeClass(51).page_count(), 1);
        assert_eq!(SizeClass(67).page_count(), 4);
    }
}
