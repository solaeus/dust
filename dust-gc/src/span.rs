use std::{
    array,
    ptr::{self, NonNull},
    sync::{
        RwLock,
        atomic::{AtomicPtr, AtomicU32, AtomicUsize, Ordering},
    },
    usize,
};

use crate::page_allocator::PAGE_SIZE;

pub const SPAN_CLASS_COUNT: usize = 136;
pub const LARGE_SIZE_MINIMUM: usize = SIZES[SIZES.len() - 1] + 1;

const SIZES: [usize; SPAN_CLASS_COUNT / 2] = [
    0, 8, 16, 24, 32, 48, 64, 80, 96, 112, 128, 144, 160, 176, 192, 208, 224, 240, 256, 288, 320,
    352, 384, 416, 448, 480, 512, 576, 640, 704, 768, 896, 1024, 1152, 1280, 1408, 1536, 1792,
    2048, 2304, 2688, 3072, 3200, 3456, 4096, 4864, 5376, 6144, 6528, 6784, 6912, 8192, 9472, 9728,
    10240, 10880, 12288, 13568, 14336, 16384, 18432, 19072, 20480, 21760, 24576, 27264, 28672,
    32768,
];

pub struct Span {
    class: SpanClass,

    start_address: usize,
    end_address: usize,

    next: Option<NonNull<Span>>,
    previous: Option<NonNull<Span>>,

    free_slots: NonNull<Bitmap>,
    free_index: u16,
    scanned_slots: NonNull<Bitmap>,
    scan_index: u16,

    generation: u32,
}

impl Span {
    pub fn new(
        start_address: usize,
        page_count: usize,
        class: SpanClass,
        free_slots: NonNull<Bitmap>,
        scanned_slots: NonNull<Bitmap>,
    ) -> Self {
        let span_size = start_address + page_count * PAGE_SIZE;
        let slot_count = span_size / class.size();

        assert!(slot_count < BitmapArena::CAPACITY);

        Self {
            class,
            start_address,
            end_address: start_address + page_count * PAGE_SIZE,
            next: None,
            previous: None,
            generation: 0,
            free_slots,
            free_index: 0,
            scanned_slots,
            scan_index: 0,
        }
    }

    pub fn allocate_slot(&mut self) -> usize {
        todo!()
    }
}

pub struct SpanClass(u8);

impl SpanClass {
    pub const LARGE_SCAN: Self = Self(0);
    pub const LARGE_NO_SCAN: Self = Self(1);

    pub fn new(size: usize, no_scan: bool) -> Option<Self> {
        if size == 0 {
            return None;
        }

        let size_index = if let Some(index) = SIZES[1..]
            .iter()
            .position(|class| *class >= size)
            .map(|index| index + 1)
        {
            index
        } else if no_scan {
            return Some(Self::LARGE_NO_SCAN);
        } else {
            return Some(Self::LARGE_SCAN);
        };
        let span_class = if no_scan {
            size_index * 2 + 1
        } else {
            size_index * 2
        };

        Some(Self(span_class as u8))
    }

    pub const fn from_index(index: usize) -> Self {
        Self(index as u8)
    }

    pub const fn index(&self) -> usize {
        self.0 as usize
    }

    pub const fn no_scan(&self) -> bool {
        self.0 & 1 != 0
    }

    pub const fn size(&self) -> usize {
        let size_index = (self.0 >> 1) as usize;

        SIZES[size_index]
    }
}

pub struct SpanList {
    first: Option<NonNull<Span>>,
    last: Option<NonNull<Span>>,
}

const BLOCK_ENTRIES: usize = 512;
const SPAN_SET_STARTING_SPINE_CAPACITY: usize = 256;

pub struct SpanSet {
    spine: RwLock<Vec<AtomicPtr<SpanSetBlock>>>,
}

impl SpanSet {
    pub fn new() -> Self {
        Self {
            spine: RwLock::new(Vec::with_capacity(SPAN_SET_STARTING_SPINE_CAPACITY)),
        }
    }
}

struct SpanSetBlock {
    popped: AtomicU32,
    spans: [AtomicPtr<Span>; BLOCK_ENTRIES],
}

#[derive(Clone, Copy)]
#[repr(C)]
pub struct Bitmap {
    bits: u8,
}

impl Bitmap {
    fn get_byte(&mut self, index: usize) -> *mut u8 {
        unsafe { (self.bits as *mut u8).add(index) }
    }

    fn get_bit(&mut self, index: usize) -> (*mut u8, u8) {
        let byte = self.get_byte(index / 8);
        let mask = 1 << (index % 8);

        (byte, mask)
    }
}

#[repr(C, align(8))]
pub struct BitmapArena {
    free: AtomicUsize,
    next: *mut u8,
    bytes: [Bitmap; Self::CAPACITY],
}

impl BitmapArena {
    const SIZE: usize = 1024 * 64;
    const HEADER_SIZE: usize = size_of::<AtomicUsize>() + size_of::<*mut u8>();
    const CAPACITY: usize = Self::SIZE - Self::HEADER_SIZE;

    const fn new() -> Self {
        Self {
            free: AtomicUsize::new(0),
            next: ptr::null_mut(),
            bytes: [Bitmap { bits: 0 }; Self::CAPACITY],
        }
    }

    fn allocate_bitmap(&self, bytes: usize) -> Option<NonNull<Bitmap>> {
        let current = self.free.load(Ordering::Relaxed);

        if current.checked_add(bytes)? > self.bytes.len() {
            return None;
        }

        let start = self.free.fetch_add(bytes, Ordering::AcqRel);

        if start.saturating_add(bytes) > self.bytes.len() {
            return None;
        }

        let pointer = {
            let base = self.bytes.as_ptr();
            let with_offset = unsafe { base.add(start) };

            unsafe { NonNull::new_unchecked(with_offset as *mut Bitmap) }
        };

        Some(pointer)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn span_class_from_index() {
        for index in 0..SPAN_CLASS_COUNT {
            let span_class = SpanClass::from_index(index);

            assert_eq!(span_class.size(), SIZES[index / 2]);
            assert_eq!(span_class.no_scan(), index % 2 == 1);
        }
    }

    #[test]
    fn span_class_new() {
        for (index, size) in (1..LARGE_SIZE_MINIMUM).enumerate() {
            let no_scan = index % 2 == 1;
            let span_class = SpanClass::new(size, no_scan).unwrap();
            let size_of_class = SIZES
                .iter()
                .find(|size_of_class| size_of_class >= &&size)
                .unwrap();

            assert_eq!(&span_class.size(), size_of_class);
            assert_eq!(span_class.no_scan(), no_scan);
        }
    }
}
