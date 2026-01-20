mod bitmap;
mod set;

pub use bitmap::{Bitmap, BitmapArena};
pub use set::SpanSet;

use std::ptr::NonNull;

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
    page_count: usize,

    start_address: usize,
    end_address: usize,

    next: Option<NonNull<Span>>,
    previous: Option<NonNull<Span>>,

    free_slots: NonNull<Bitmap>,
    free_index: u16,
    scanned_slots: NonNull<Bitmap>,
    scan_index: u16,

    full_slots: u16,
    pub generation: u32,
}

impl Span {
    pub fn new(
        start_address: usize,
        page_count: usize,
        class: SpanClass,
        free_slots: NonNull<Bitmap>,
        scanned_slots: NonNull<Bitmap>,
    ) -> Self {
        Self {
            class,
            page_count,
            start_address,
            end_address: start_address + page_count * PAGE_SIZE,
            next: None,
            previous: None,
            free_slots,
            free_index: 0,
            scanned_slots,
            scan_index: 0,
            full_slots: 0,
            generation: 0,
        }
    }

    pub fn allocate_slot(&mut self) -> Option<NonNull<u8>> {
        let slot_count = if self.class.is_large() {
            1
        } else {
            self.class.size() / self.page_count
        };

        if self.free_index as usize == slot_count {
            return None;
        }

        todo!()
    }

    pub fn sweep(&mut self) {}

    pub fn is_full(&self) -> bool {
        self.full_slots == self.slot_count() as u16
    }

    pub fn full_slots(&self) -> u16 {
        self.full_slots
    }

    pub fn slot_count(&self) -> usize {
        (self.start_address + self.page_count * PAGE_SIZE) / self.class.size()
    }
}

#[derive(Clone, Copy, PartialEq)]
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

    pub const fn is_large(&self) -> bool {
        matches!(*self, Self::LARGE_SCAN | Self::LARGE_NO_SCAN)
    }
}

pub struct SpanList {
    first: Option<NonNull<Span>>,
    last: Option<NonNull<Span>>,
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
