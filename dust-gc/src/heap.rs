use std::{array, ptr::NonNull, sync::Arc};

use parking_lot::Mutex;

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
            centrals: array::from_fn(|index| Central::new(SpanClass::from_index(index))),
            mutex: Mutex::new(HeapLocked {
                arenas: Vec::with_capacity(1),
                page_allocator: PageAllocator::new(),
                all_spans: Vec::new(),
                arenas_mark_snapshot: vec![0].into_boxed_slice(),
                arenas_sweep_snapshot: vec![0].into_boxed_slice(),
            }),
        }))
    }

    pub fn create_thread_cache(&mut self) -> ThreadCache {
        let spans = array::from_fn(|index| self.0.allocate_span(SpanClass::from_index(index)));

        ThreadCache::new(self.clone(), spans)
    }
}

pub struct HeapShared {
    centrals: [Central; SPAN_CLASS_COUNT],

    mutex: Mutex<HeapLocked>,
}

impl HeapShared {
    pub fn allocate_span(&self, class: SpanClass) -> NonNull<Span> {
        // let central = &self.centrals[class.index()];

        todo!()
    }
}

struct HeapLocked {
    arenas: Vec<Arena>,

    page_allocator: PageAllocator,

    all_spans: Vec<Span>,

    arenas_mark_snapshot: Box<[usize]>,

    arenas_sweep_snapshot: Box<[usize]>,
}

impl HeapLocked {
    fn grow(&mut self, page_count: usize) -> usize {
        let arena_count = page_count.div_ceil(PAGES_PER_ARENA);

        for _ in 0..arena_count {
            let arena = Arena::new();

            self.page_allocator.grow(arena.base, ARENA_SIZE);
            self.arenas.push(Arena::new());
        }

        arena_count * ARENA_SIZE
    }
}
