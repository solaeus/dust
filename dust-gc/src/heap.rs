use std::{array, ptr::NonNull, sync::Arc};

use parking_lot::Mutex;

use crate::{
    ThreadCache,
    arena::{ARENA_SIZE, Arena, PAGES_PER_ARENA},
    central::Central,
    page_allocator::{OffsetAddress, PageAllocator},
    span::{BitmapArena, SPAN_CLASS_COUNT, Span, SpanClass},
};

#[derive(Clone)]
pub struct Heap(Arc<HeapShared>);

impl Heap {
    pub fn new(free_memory_threshold: f32) -> Self {
        Heap(Arc::new(HeapShared {
            lock: Mutex::new(HeapLocked {
                arenas: Vec::with_capacity(1),
                page_allocator: PageAllocator::new(),
                all_spans: Vec::new(),
                bitmap_arenas: Vec::new(),
            }),
            centrals: array::from_fn(|_| Central::new()),
            generation: 0,
        }))
    }

    pub fn thread_cache(&mut self) -> ThreadCache {
        ThreadCache::new(Arc::clone(&self.0))
    }
}

pub struct HeapShared {
    pub lock: Mutex<HeapLocked>,

    centrals: [Central; SPAN_CLASS_COUNT],

    generation: u32,
}

impl HeapShared {
    pub fn get_free_span(&self, class: SpanClass) -> NonNull<Span> {
        if let Some(span_pointer) = self.centrals[class.index()].get_span() {
            let span = unsafe { span_pointer.as_ref() };

            if !span.is_full() {
                return span_pointer;
            }
        }

        let new_span = self.lock.lock().allocate_span(class);

        self.centrals[class.index()].insert_span(new_span, self.generation);

        new_span
    }
}

pub struct HeapLocked {
    arenas: Vec<Arena>,

    page_allocator: PageAllocator,

    all_spans: Vec<usize>,

    bitmap_arenas: Vec<BitmapArena>,
}

impl HeapLocked {
    pub fn find_pages(&self, page_count: usize) -> Option<(usize, OffsetAddress)> {
        todo!()
    }

    pub fn grow(&mut self, page_count: usize) -> usize {
        let arena_count = page_count.div_ceil(PAGES_PER_ARENA);

        for _ in 0..arena_count {
            let arena = Arena::new();

            self.page_allocator.grow(arena.base(), ARENA_SIZE);
            self.arenas.push(Arena::new());
        }

        arena_count * ARENA_SIZE
    }

    pub fn allocate_span(&mut self, class: SpanClass) -> NonNull<Span> {
        todo!()
    }
}
