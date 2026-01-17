use std::{
    array,
    ptr::NonNull,
    sync::{Arc, Mutex},
};

use crate::{
    ThreadCache,
    arena::{ARENA_SIZE, Arena, PAGES_PER_ARENA},
    central::Central,
    page_allocator::PageAllocator,
    span::{SPAN_CLASS_COUNT, Span, SpanClass},
};

#[derive(Clone)]
pub struct Heap(Arc<HeapInner>);

impl Heap {
    pub fn new(free_memory_threshold: f32) -> Self {
        Heap(Arc::new(HeapInner {
            centrals: array::from_fn(|index| Central::new(SpanClass::from_index(index))),
            mutex: Mutex::new(HeapLocked {
                arenas: Vec::with_capacity(1),
                page_allocator: PageAllocator::new(),
                all_spans: Vec::new(),
            }),
            arenas_mark_snapshot: Vec::with_capacity(1),
            arenas_sweep_snapshot: Vec::with_capacity(1),
            free_memory_threshold,
        }))
    }

    pub fn create_thread_cache(&mut self) -> ThreadCache {
        let spans = array::from_fn(|index| self.0.allocate_span(SpanClass::from_index(index)));

        ThreadCache::new(self.clone(), spans)
    }
}

pub struct HeapInner {
    centrals: [Central; SPAN_CLASS_COUNT],

    mutex: Mutex<HeapLocked>,

    arenas_mark_snapshot: Vec<usize>,

    arenas_sweep_snapshot: Vec<usize>,

    free_memory_threshold: f32,
}

impl HeapInner {
    pub fn allocate_span(&self, class: SpanClass) -> NonNull<Span> {
        let central = &self.centrals[class.index()];
        let span = central.find_span_with_free_slot();

        todo!()
    }
}

struct HeapLocked {
    arenas: Vec<Arena>,

    page_allocator: PageAllocator,

    all_spans: Vec<Span>,
}

impl HeapLocked {
    fn grow(&mut self, page_count: usize) -> usize {
        let arena_count = page_count.div_ceil(PAGES_PER_ARENA);

        for _ in 0..arena_count {
            self.arenas.push(Arena::new());
        }

        arena_count * ARENA_SIZE
    }
}
