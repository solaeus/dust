#![feature(allocator_api)]

mod arena;
mod block;
mod block_cache;
mod global_heap;
mod page_allocator;
mod thread_heap;

pub use crate::global_heap::GlobalHeap;
pub use crate::thread_heap::ThreadHeap;
