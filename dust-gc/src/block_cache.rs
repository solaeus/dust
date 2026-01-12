use crate::{block::BlockList, classes::SpanClass};

pub struct BlockCache {
    span_class: SpanClass,
    partial_swept: BlockList,
    partial_unswept: BlockList,
    full_swept: BlockList,
    full_unswept: BlockList,
}

impl BlockCache {
    pub fn new(span_class: SpanClass) -> Self {
        BlockCache {
            span_class,
            partial_swept: BlockList::new(),
            partial_unswept: BlockList::new(),
            full_swept: BlockList::new(),
            full_unswept: BlockList::new(),
        }
    }
}
