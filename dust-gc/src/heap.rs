use std::{array, ptr::NonNull, sync::Arc};

use parking_lot::{Mutex, MutexGuard};

use crate::{
    ThreadCache,
    arena::{ARENA_SIZE, Arena, PAGES_PER_ARENA},
    central::Central,
    fixed_allocator::FixedAllocator,
    page_allocator::PageAllocator,
    span::{SPAN_CLASS_COUNT, Span, SpanClass},
};

#[derive(Clone)]
pub struct Heap(Arc<HeapShared>);

impl Heap {
    pub fn new(free_memory_threshold: f32, span_allocator: &FixedAllocator<Span>) -> Self {
        Heap(Arc::new(HeapShared {
            // centrals: array::from_fn(|index| Central::new(SpanClass::from_index(index))),
            mutex: Mutex::new(HeapLocked {
                arenas: Vec::with_capacity(1),
                page_allocator: PageAllocator::new(),
                all_spans: Vec::new(),
                span_allocator: FixedAllocator::new(),
                thread_cache_allocator: FixedAllocator::new(),
            }),
        }))
    }

    pub fn thread_cache(&mut self) -> NonNull<ThreadCache> {
        let heap_locked = self.0.mutex.lock();
        let spans = array::from_fn(|index| {
            let class = SpanClass::from_index(1);
            let span_memory_pointer = heap_locked
                .span_allocator
                .allocate()
                .as_ptr()
                .cast::<Span>();
            let span = Span::new(start_address, page_count, class, free_slots, scanned_slots);
        });
        let cache_memory_pointer = heap_locked
            .thread_cache_allocator
            .allocate()
            .as_ptr()
            .cast::<ThreadCache>();
        let cache = ThreadCache::new(Arc::clone(&self.0), spans);

        unsafe { cache_memory_pointer.write(cache) };
        unsafe { NonNull::new_unchecked(cache_memory_pointer) }
    }
}

pub struct HeapShared {
    pub mutex: Mutex<HeapLocked>,
}

pub struct HeapLocked {
    arenas: Vec<Arena>,

    page_allocator: PageAllocator,

    all_spans: Vec<NonNull<Span>>,

    span_allocator: FixedAllocator<Span>,

    thread_cache_allocator: FixedAllocator<ThreadCache>,
}

impl HeapLocked {
    pub fn find_pages(&self, page_count: usize) -> Option<(usize, OffetAddress)> {
        self.page_allocator.find(page_count)
    }

    pub fn grow_pages(&mut self, page_count: usize) -> usize {
        let arena_count = page_count.div_ceil(PAGES_PER_ARENA);

        for _ in 0..arena_count {
            let arena = Arena::new();

            self.page_allocator.grow(arena.base, ARENA_SIZE);
            self.arenas.push(Arena::new());
        }

        arena_count * ARENA_SIZE
    }
}
