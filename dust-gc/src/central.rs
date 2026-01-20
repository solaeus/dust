use std::ptr::NonNull;

use crate::span::{Span, SpanSet};

pub struct Central {
    pub partial_swept: SpanSet,
    pub partial_unswept: SpanSet,
    pub full_swept: SpanSet,
    pub full_unswept: SpanSet,
}

impl Central {
    pub fn new() -> Self {
        Self {
            partial_swept: SpanSet::new(),
            partial_unswept: SpanSet::new(),
            full_swept: SpanSet::new(),
            full_unswept: SpanSet::new(),
        }
    }

    pub fn get_span(&self) -> Option<NonNull<Span>> {
        self.partial_swept
            .pop()
            .or_else(|| self.partial_unswept.pop())
            .or_else(|| self.full_unswept.pop())
            .or_else(|| self.full_swept.pop())
    }

    pub fn insert_span(&self, mut span_pointer: NonNull<Span>, generation: u32) {
        let span = unsafe { span_pointer.as_mut() };
        let is_stale = span.generation == generation + 1;

        if is_stale {
            span.generation = generation - 1;

            span.sweep();
        } else {
            span.generation = generation;

            if span.slot_count() - span.full_slots() as usize > 0 {
                self.partial_swept.push(span_pointer);
            } else {
                self.full_swept.push(span_pointer);
            }
        }
    }
}

unsafe impl Send for Central {}
unsafe impl Sync for Central {}
