use std::ptr::NonNull;

use bitvec::vec::BitVec;

use crate::page_allocator::PAGE_SIZE;

pub struct Block {
    size_class: SizeClass,

    start_address: NonNull<u8>,

    slot_size: usize,
    slot_count: usize,
    slot_tags: BitVec,
    page_count: usize,

    free_index: usize,
    allocated_count: usize,

    heap_bits: Option<BitVec>,

    next: Option<NonNull<Block>>,
    prev: Option<NonNull<Block>>,
}

impl Block {
    pub fn new(
        start_address: NonNull<u8>,
        page_count: usize,
        size_class: SizeClass,
    ) -> NonNull<Block> {
        let slot_size = size_class.size();
        let total_bytes = page_count * PAGE_SIZE;
        let slot_count = total_bytes / slot_size;
        let block = Block {
            start_address,
            page_count,
            size_class,
            slot_size,
            slot_count,
            slot_tags: BitVec::repeat(false, slot_count),
            free_index: 0,
            allocated_count: 0,
            heap_bits: None,
            next: None,
            prev: None,
        };
        let pointer = Box::into_raw(Box::new(block));

        unsafe { NonNull::new_unchecked(pointer) }
    }

    pub fn reuse(&mut self, start_address: NonNull<u8>, page_count: usize, size_class: SizeClass) {
        let slot_size = size_class.size();
        let total_bytes = page_count * PAGE_SIZE;
        let slot_count = total_bytes / slot_size;

        self.start_address = start_address;
        self.page_count = page_count;
        self.size_class = size_class;
        self.slot_size = slot_size;
        self.slot_count = slot_count;
        self.slot_tags = BitVec::repeat(false, slot_count);
        self.free_index = 0;
        self.allocated_count = 0;
        self.heap_bits = None;
        self.next = None;
        self.prev = None;
    }

    pub fn allocate_slot(&mut self) -> Option<NonNull<u8>> {
        debug_assert!(self.allocated_count <= self.slot_count);

        if self.allocated_count == self.slot_count {
            return None;
        }

        let index = self.slot_tags.first_zero()?;

        self.slot_tags.set(index, true);
        self.allocated_count += 1;
        self.free_index = index + 1;

        let offset = index * self.slot_size;
        let slot_address = unsafe { self.start_address.as_ptr().add(offset) };
        let slot_pointer = unsafe { NonNull::new_unchecked(slot_address) };

        Some(slot_pointer)
    }

    pub fn free_slot(&mut self, slot_pointer: NonNull<u8>) {
        let base_address = self.start_address.as_ptr() as usize;
        let slot_address = slot_pointer.as_ptr() as usize;
        let offset = slot_address - base_address;

        debug_assert!(slot_address >= base_address);
        debug_assert!(offset % self.slot_size == 0);

        let index = offset / self.slot_size;

        assert!(index < self.slot_count);

        if self.slot_tags[index] {
            self.slot_tags.set(index, false);
            self.allocated_count -= 1;

            if index < self.free_index {
                self.free_index = index;
            }
        }
    }

    pub const fn is_empty(&self) -> bool {
        self.allocated_count == 0
    }

    pub const fn is_full(&self) -> bool {
        self.allocated_count == self.slot_count
    }
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

    pub const fn index(&self) -> usize {
        self.0 as usize
    }

    pub const fn size(&self) -> usize {
        Self::SIZES[self.0 as usize]
    }

    pub const fn page_count(&self) -> usize {
        self.size().div_ceil(PAGE_SIZE)
    }

    pub fn is_large(&self) -> bool {
        self == &Self::LARGE
    }

    pub const fn is_tiny(&self) -> bool {
        matches!(self.0, 1 | 2)
    }
}

pub struct BlockList {
    pub head: Option<NonNull<Block>>,
    pub tail: Option<NonNull<Block>>,
}

impl BlockList {
    pub fn new() -> Self {
        Self {
            head: None,
            tail: None,
        }
    }

    pub const fn is_empty(&self) -> bool {
        self.head.is_none()
    }

    pub fn push_front(&mut self, block_pointer: NonNull<Block>) {
        let block = unsafe { &mut *block_pointer.as_ptr() };
        block.prev = None;
        block.next = self.head;

        if let Some(old_head_pointer) = self.head {
            let old_head = unsafe { &mut *old_head_pointer.as_ptr() };

            old_head.prev = Some(block_pointer);
        } else {
            self.tail = Some(block_pointer);
        }

        self.head = Some(block_pointer);
    }

    pub fn push_back(&mut self, block_pointer: NonNull<Block>) {
        let block = unsafe { &mut *block_pointer.as_ptr() };
        block.next = None;
        block.prev = self.tail;

        if let Some(old_tail_pointer) = self.tail {
            let old_tail = unsafe { &mut *old_tail_pointer.as_ptr() };

            old_tail.next = Some(block_pointer);
        } else {
            self.head = Some(block_pointer);
        }

        self.tail = Some(block_pointer);
    }

    pub fn pop_front(&mut self) -> Option<NonNull<Block>> {
        let head_pointer = self.head?;
        let head = unsafe { &mut *head_pointer.as_ptr() };
        self.head = head.next;

        if let Some(new_head_pointer) = self.head {
            let new_head = unsafe { &mut *new_head_pointer.as_ptr() };

            new_head.prev = None;
        } else {
            self.tail = None;
        }

        head.next = None;
        head.prev = None;

        Some(head_pointer)
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

    #[test]
    fn block_list_push_pop() {
        let block1 = Block::new(NonNull::dangling(), 1, SizeClass(1));
        let block2 = Block::new(NonNull::dangling(), 1, SizeClass(2));
        let block3 = Block::new(NonNull::dangling(), 1, SizeClass(3));

        let mut list = BlockList::new();

        assert!(list.is_empty());

        list.push_back(block1);
        list.push_back(block2);
        list.push_front(block3);

        assert!(!list.is_empty());
        assert_eq!(list.pop_front(), Some(block3));
        assert_eq!(list.pop_front(), Some(block1));
        assert_eq!(list.pop_front(), Some(block2));
        assert_eq!(list.pop_front(), None);
        assert!(list.is_empty());
    }
}
