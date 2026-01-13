use crate::block::{BlockList, SizeClass};

pub struct BlockCache {
    size_class: SizeClass,
    partial_swept: BlockList,
    partial_unswept: BlockList,
    full_swept: BlockList,
    full_unswept: BlockList,
}

impl BlockCache {
    pub fn new(size_class: SizeClass) -> Self {
        BlockCache {
            size_class,
            partial_swept: BlockList::new(),
            partial_unswept: BlockList::new(),
            full_swept: BlockList::new(),
            full_unswept: BlockList::new(),
        }
    }
}
