use crate::span::{SpanClass, SpanSet};

pub struct Central {
    pub span_class: SpanClass,
    pub partial_swept: SpanSet,
    pub partial_unswept: SpanSet,
    pub full_swept: SpanSet,
    pub full_unswept: SpanSet,
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
}
