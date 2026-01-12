#![feature(allocator_api)]

mod arena;
mod block;
mod block_cache;
mod classes;
mod global_heap;
mod page_allocator;
mod page_cache;
mod radix_tree;
mod thread_heap;

pub use crate::global_heap::GlobalHeap;
pub use crate::thread_heap::ThreadHeap;

const PAGE_SIZE: usize = 8192;
const ARENA_SIZE: usize = 64 * 1024 * 1024;
