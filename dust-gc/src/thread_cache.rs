use std::ptr::NonNull;

use crate::{
    Heap,
    page_allocator::PAGE_SIZE,
    span::{SPAN_CLASS_COUNT, Span, SpanClass},
};

pub struct ThreadCache {
    heap: Heap,
    page_cache: PageCache,
    spans: [NonNull<Span>; SPAN_CLASS_COUNT],
}

impl ThreadCache {
    pub(crate) fn new(heap: Heap, spans: [NonNull<Span>; SPAN_CLASS_COUNT]) -> Self {
        Self {
            heap,
            page_cache: PageCache::default(),
            spans,
        }
    }

    pub fn allocate(&mut self, size: usize, no_scan: bool) -> Option<NonNull<u8>> {
        let span_class = SpanClass::new(size, no_scan)?;
        let span = unsafe { self.spans[span_class.index()].as_mut() };

        if let Some(address) = span.allocate_slot() {
            let pointer = unsafe { NonNull::new_unchecked(address as *mut u8) };

            return Some(pointer);
        }

        todo!()
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
