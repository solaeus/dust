use std::{array, ptr::NonNull, sync::Arc};

use crate::{
    heap::HeapShared,
    page_allocator::PAGE_SIZE,
    span::{SPAN_CLASS_COUNT, Span, SpanClass},
};

pub struct ThreadCache {
    heap: Arc<HeapShared>,

    spans: [Option<NonNull<Span>>; SPAN_CLASS_COUNT],
}

impl ThreadCache {
    pub(crate) fn new(heap: Arc<HeapShared>) -> Self {
        Self {
            heap,
            spans: [None; SPAN_CLASS_COUNT],
        }
    }

    pub fn allocate_slot(&mut self, size: usize, no_scan: bool) -> Option<NonNull<u8>> {
        let span_class = SpanClass::new(size, no_scan)?;
        let span = if let Some(mut span) = self.spans[span_class.index()] {
            unsafe { span.as_mut() }
        } else {
            let mut span = self.heap.get_free_span(span_class);

            unsafe { span.as_mut() }
        };

        span.allocate_slot()
    }
}

struct SpanCache {
    buffer: [Option<NonNull<Span>>; 128],
    length: usize,
}

impl SpanCache {
    fn new() -> Self {
        Self {
            buffer: [None; 128],
            length: 0,
        }
    }
}

#[derive(Default)]
struct PageCache {
    base: usize,
    free_pages: u64,
    scavenged_pages: u64,
}

impl PageCache {
    /// Allocates `count` pages and returns their base address and number of scavenged bytes, or
    /// None if no pages are available.
    fn allocate_pages(&mut self, count: usize) -> Option<(usize, usize)> {
        if self.is_empty() {
            return None;
        }

        if count == 1 {
            let index = self.free_pages.trailing_zeros() as usize;
            let base = self.base + index * PAGE_SIZE;
            let scavenged = ((self.scavenged_pages >> index) & 1) as usize * PAGE_SIZE;

            self.free_pages &= !(1 << index); // set bit at index to mark page as non-free
            self.scavenged_pages &= !(1 << index); // clear bit at index to mark page as unscavenged

            return Some((base, scavenged));
        }

        let index = self.free_pages.lowest_one()? as usize;
        let base = self.base + index * PAGE_SIZE;
        let mask = ((1 << count) - 1) << index;
        let scavenged = (self.scavenged_pages & mask).count_ones() as usize * PAGE_SIZE;

        self.free_pages &= !mask; // set bits to mark pages as non-free
        self.scavenged_pages &= !mask; // clear bits to mark pages as non-scavenged

        Some((base, scavenged))
    }

    fn is_empty(&self) -> bool {
        self.free_pages == 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn allocate_pages_from_page_cache() {
        let mut page_cache = PageCache {
            base: 0,
            free_pages: u64::MAX,
            scavenged_pages: 0,
        };

        let (base, scavenged) = page_cache.allocate_pages(1).unwrap();

        assert_eq!(base, 0);
        assert_eq!(scavenged, 0);
        assert_eq!(page_cache.free_pages.leading_ones(), 63);
        assert_eq!(page_cache.free_pages.trailing_zeros(), 1);

        let (base, scavenged) = page_cache.allocate_pages(10).unwrap();

        assert_eq!(base, PAGE_SIZE);
        assert_eq!(scavenged, 0);
    }
}
