use std::ptr::NonNull;

use crate::span::{Span, SpanClass, SpanSet};

pub struct Central {
    span_class: SpanClass,
    partial_swept: SpanSet,
    partial_unswept: SpanSet,
    full_swept: SpanSet,
    full_unswept: SpanSet,
}

impl Central {
    pub fn new(span_class: SpanClass) -> Self {
        Self {
            span_class,
            partial_swept: SpanSet::new(),
            partial_unswept: SpanSet::new(),
            full_swept: SpanSet::new(),
            full_unswept: SpanSet::new(),
        }
    }

    pub fn find_span_with_free_slot(&self) -> NonNull<Span> {
        todo!()
    }
}
