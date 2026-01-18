use std::{
    marker::PhantomData,
    mem::MaybeUninit,
    ptr::{self, NonNull},
    sync::atomic::AtomicPtr,
};

use crate::{
    lock_free_stack::{LockFreeNode, LockFreeStack},
    region::Region,
};

const CHUNK_SIZE: usize = 1024 * 16;

pub struct FixedAllocator<T> {
    free_list: *mut u8,

    current_chunk: *mut u8,
    curret_chunk_remaining: usize,

    chunks: LockFreeStack<Chunk>,

    used: usize,
    chunk_capacity: usize,
    element_size: usize,
    element_align: usize,

    _phantom: PhantomData<T>,
}

impl<T> FixedAllocator<T> {
    pub fn new() -> Self {
        let element_align = align_of::<T>().max(align_of::<*mut u8>());
        let unaligned_size = size_of::<T>().max(size_of::<*mut u8>());
        let element_size = (unaligned_size + element_align - 1) & !(element_align - 1);
        let capacity = CHUNK_SIZE / element_size;

        Self {
            free_list: ptr::null_mut(),
            current_chunk: ptr::null_mut(),
            curret_chunk_remaining: 0,
            chunks: LockFreeStack::new(),
            chunk_capacity: capacity,
            used: 0,
            element_size,
            element_align,
            _phantom: PhantomData,
        }
    }

    pub fn allocate(&mut self) -> NonNull<MaybeUninit<T>> {
        if self.free_list.is_null() {
            if self.curret_chunk_remaining < self.element_size {
                self.refill();
            }

            self.current_chunk = unsafe { self.current_chunk.add(self.element_size) };
            self.curret_chunk_remaining -= self.element_size;
            self.used += self.element_size;

            return unsafe { NonNull::new_unchecked(self.current_chunk.cast::<MaybeUninit<T>>()) };
        }

        self.free_list = unsafe { *(self.free_list.cast::<*mut u8>()) };
        self.used += self.element_size;

        unsafe { NonNull::new_unchecked(self.free_list.cast::<MaybeUninit<T>>()) }
    }

    pub fn refill(&mut self) {
        let region = Region::new(self.chunk_capacity)
            .expect("Failed to allocate memory region for FixedAllocator.");
        let base = region.pointer().as_ptr();
        let size = region.size();

        debug_assert!(size >= self.element_size);
        debug_assert!(base as usize % self.element_align == 0);

        let new_chunk = Box::new(Chunk {
            next: AtomicPtr::null(),
            region,
        });
        let pointer = unsafe { NonNull::new_unchecked(Box::into_raw(new_chunk)) };

        self.chunks.push(pointer);

        self.current_chunk = base;
        self.curret_chunk_remaining = size;
    }
}

struct Chunk {
    next: AtomicPtr<Chunk>,
    region: Region,
}

impl LockFreeNode for Chunk {
    fn link(&self) -> &AtomicPtr<Self> {
        &self.next
    }
}
