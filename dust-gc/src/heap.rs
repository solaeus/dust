use std::{array, ptr::NonNull, sync::Mutex};

use crate::{
    arena::Arena,
    central::Central,
    page_allocator::PageAllocator,
    span::{SPAN_CLASS_COUNT, Span, SpanClass},
};

pub struct Heap {
    arenas: Vec<Arena>,

    centrals: [Central; SPAN_CLASS_COUNT],

    page_allocator: Mutex<PageAllocator>,

    all_spans: Vec<NonNull<Span>>,

    arenas_mark_snapshot: Vec<usize>,

    arenas_sweep_snapshot: Vec<usize>,

    free_memory_threshold: f32,
}

impl Heap {
    pub fn new(free_memory_threshold: f32) -> Self {
        Heap {
            arenas: Vec::with_capacity(1),
            centrals: array::from_fn(|index| Central::new(SpanClass::from_index(index))),
            page_allocator: Mutex::new(PageAllocator::new()),
            all_spans: Vec::new(),
            arenas_mark_snapshot: Vec::with_capacity(1),
            arenas_sweep_snapshot: Vec::with_capacity(1),
            free_memory_threshold,
        }
    }

    pub fn allocate_span(&mut self, page_count: usize, class: SpanClass) -> NonNull<Span> {
        todo!()
    }
}
