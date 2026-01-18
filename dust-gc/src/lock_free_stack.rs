use std::{
    hint,
    marker::PhantomData,
    ptr::{self, NonNull},
    sync::atomic::{AtomicPtr, Ordering},
};

use crossbeam_epoch::{Atomic, Owned, pin};

#[cfg(any(
    target_pointer_width = "32",
    all(
        target_pointer_width = "64",
        any(
            target_arch = "x86_64",
            target_arch = "aarch64",
            target_arch = "riscv64",
            target_arch = "loongarch64"
        )
    )
))]
type PlatformPackedAtomic = packed_atomic_64::PackedAtomic64;

#[cfg(all(
    target_pointer_width = "64",
    not(any(
        target_arch = "x86_64",
        target_arch = "aarch64",
        target_arch = "riscv64",
        target_arch = "loongarch64"
    )),
    target_has_atomic = "128"
))]
type PlatformPackedAtomic = packed_atomic_128::PackedAtomic128;

pub type LockFreeStack<T> = Treiber<PlatformPackedAtomic, T>;

struct Treiber<A, T> {
    head: Padded<A>,
    _phantom: PhantomData<*mut T>,
}

impl<Packed: PackedAtomic, T: LockFreeNode> Treiber<Packed, T> {
    pub fn new() -> Self {
        Self {
            head: Padded(Packed::default()),
            _phantom: PhantomData,
        }
    }

    pub fn push(&self, item: NonNull<T>) {
        let item_pointer = item.as_ptr();

        loop {
            let old = self.head.0.load(Ordering::Acquire);
            let (old_pointer, old_count) = Packed::unpack(old);
            let old_pointer = old_pointer as *mut T;
            let item = unsafe { &*item_pointer };

            item.link().store(old_pointer);

            let new = Packed::pack(item_pointer as usize, old_count.wrapping_add(1));

            match self
                .head
                .0
                .compare_exchange_weak(old, new, Ordering::Release, Ordering::Acquire)
            {
                Ok(_) => return,
                Err(_) => hint::spin_loop(),
            }
        }
    }

    pub fn pop(&self) -> Option<NonNull<T>> {
        loop {
            let old = self.head.0.load(Ordering::Acquire);
            let (old_pointer, old_count) = Packed::unpack(old);
            let old_pointer = old_pointer as *mut T;

            if old_pointer.is_null() {
                return None;
            }

            let old_item = unsafe { &*old_pointer };
            let next = old_item.link().load();
            let new = Packed::pack(next as usize, old_count.wrapping_add(1));

            match self
                .head
                .0
                .compare_exchange_weak(old, new, Ordering::AcqRel, Ordering::Acquire)
            {
                Ok(_) => {
                    old_item.link().store(ptr::null_mut());

                    let popped = unsafe { NonNull::new_unchecked(old_pointer) };

                    return Some(popped);
                }
                Err(_) => hint::spin_loop(),
            }
        }
    }
}

unsafe impl<A, T: LockFreeNode + Send> Send for Treiber<A, T> {}
unsafe impl<A, T: LockFreeNode + Send> Sync for Treiber<A, T> {}

pub trait LockFreeNode: Sized {
    fn link(&self) -> &Link<Self>;
}

#[repr(C)]
pub struct Link<T> {
    next: AtomicPtr<T>,
}

impl<T> Link<T> {
    pub fn new() -> Self {
        Self {
            next: AtomicPtr::null(),
        }
    }

    fn load(&self) -> *mut T {
        self.next.load(Ordering::Relaxed)
    }

    fn store(&self, pointer: *mut T) {
        self.next.store(pointer, Ordering::Relaxed);
    }
}

#[repr(align(64))]
struct Padded<T>(T);

trait PackedAtomic: Default {
    type Word: Copy;

    fn pack(pointer: usize, count: u64) -> Self::Word;
    fn unpack(word: Self::Word) -> (usize, u64);
    fn load(&self, ordering: Ordering) -> Self::Word;
    fn compare_exchange_weak(
        &self,
        old: Self::Word,
        new: Self::Word,
        success: Ordering,
        failure: Ordering,
    ) -> Result<Self::Word, Self::Word>;
}

mod packed_atomic_64 {
    use super::*;

    use std::{sync::atomic::AtomicU64, u64};

    const FIELD_BITS: u32 = if cfg!(target_pointer_width = "32") {
        32
    } else {
        48
    };
    const POINTER_MASK: u64 = (1u64 << FIELD_BITS) - 1;

    #[derive(Default)]
    pub struct PackedAtomic64(AtomicU64);

    impl PackedAtomic for PackedAtomic64 {
        type Word = u64;

        fn pack(pointer: usize, count: u64) -> Self::Word {
            (pointer as u64 & POINTER_MASK) | (count << FIELD_BITS)
        }

        fn unpack(word: Self::Word) -> (usize, u64) {
            let pointer = (word & FIELD_BITS as u64) as usize;
            let count = (word >> FIELD_BITS) as u64;

            (pointer, count)
        }

        fn load(&self, ordering: Ordering) -> Self::Word {
            self.0.load(ordering)
        }

        fn compare_exchange_weak(
            &self,
            old: Self::Word,
            new: Self::Word,
            success: Ordering,
            failure: Ordering,
        ) -> Result<Self::Word, Self::Word> {
            self.0.compare_exchange_weak(old, new, success, failure)
        }
    }
}

#[cfg(target_has_atomic = "128")]
mod packed_atomic_128 {
    use std::sync::atomic::AtomicU128;

    use super::*;

    #[derive(Default)]
    pub struct PackedAtomic128(AtomicU128);

    impl PackedAtomic for PackedAtomic128 {
        type Word = u128;

        fn pack(pointer: usize, count: u64) -> Self::Word {
            (pointer as u128) | ((count as u128) << 64)
        }

        fn unpack(word: Self::Word) -> (usize, u64) {
            let pointer = word as usize;
            let count = (word >> 64) as u64;

            (pointer, count)
        }

        fn load(&self, ordering: Ordering) -> Self::Word {
            self.0.load(ordering)
        }

        fn compare_exchange_weak(
            &self,
            old: Self::Word,
            new: Self::Word,
            success: Ordering,
            failure: Ordering,
        ) -> Result<Self::Word, Self::Word> {
            self.0.compare_exchange_weak(old, new, success, failure)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{
        boxed::Box,
        sync::{
            Arc, Mutex,
            atomic::{AtomicUsize, Ordering},
        },
        thread,
    };

    #[repr(C)]
    struct Node {
        link: Link<Node>,
        v: usize,
    }

    impl LockFreeNode for Node {
        #[inline]
        fn link(&self) -> &Link<Self> {
            &self.link
        }
    }

    fn boxed_node(v: usize) -> usize {
        let b = Box::new(Node {
            link: Link::new(),
            v,
        });
        Box::into_raw(b) as usize
    }

    unsafe fn drop_node(p: usize) {
        drop(Box::from_raw(p as *mut Node));
    }

    #[test]
    fn lifo_single_thread_manual_reclaim() {
        let s = LockFreeStack::<Node>::new();
        let a = unsafe { NonNull::new_unchecked(boxed_node(1) as *mut Node) };
        let b = unsafe { NonNull::new_unchecked(boxed_node(2) as *mut Node) };
        let c = unsafe { NonNull::new_unchecked(boxed_node(3) as *mut Node) };

        unsafe {
            s.push(a);
            s.push(b);
            s.push(c);

            let n1 = s.pop().unwrap();
            assert_eq!(n1.as_ref().v, 3);
            drop_node(n1.as_ptr() as usize);

            let n2 = s.pop().unwrap();
            assert_eq!(n2.as_ref().v, 2);
            drop_node(n2.as_ptr() as usize);

            let n3 = s.pop().unwrap();
            assert_eq!(n3.as_ref().v, 1);
            drop_node(n3.as_ptr() as usize);

            assert!(s.pop().is_none());
        }
    }

    #[test]
    fn concurrent_push_pop_manual_quiescence_then_reclaim() {
        let s = Arc::new(LockFreeStack::<Node>::new());
        let total = 20_000usize;
        let producers = 4usize;
        let consumers = 4usize;

        let next_id = Arc::new(AtomicUsize::new(0));
        let popped = Arc::new(AtomicUsize::new(0));

        let popped_ptrs = Arc::new(Mutex::new(Vec::<usize>::with_capacity(total)));
        let popped_vals = Arc::new(Mutex::new(Vec::<usize>::with_capacity(total)));

        let mut joins = Vec::new();

        for _ in 0..producers {
            let s = Arc::clone(&s);
            let next_id = Arc::clone(&next_id);
            joins.push(thread::spawn(move || {
                loop {
                    let id = next_id.fetch_add(1, Ordering::AcqRel);
                    if id >= total {
                        break;
                    }
                    let p = boxed_node(id);
                    let nn = unsafe { NonNull::new_unchecked(p as *mut Node) };
                    unsafe { s.push(nn) };
                }
            }));
        }

        for _ in 0..consumers {
            let s = Arc::clone(&s);
            let popped = Arc::clone(&popped);
            let popped_ptrs = Arc::clone(&popped_ptrs);
            let popped_vals = Arc::clone(&popped_vals);
            joins.push(thread::spawn(move || {
                while popped.load(Ordering::Acquire) < total {
                    let n = unsafe { s.pop() };
                    if let Some(n) = n {
                        let v = unsafe { n.as_ref().v };
                        let p = n.as_ptr() as usize;
                        {
                            popped_ptrs.lock().unwrap().push(p);
                            popped_vals.lock().unwrap().push(v);
                        }
                        popped.fetch_add(1, Ordering::AcqRel);
                    } else {
                        thread::yield_now();
                    }
                }
            }));
        }

        for j in joins {
            j.join().unwrap();
        }

        assert_eq!(popped.load(Ordering::Acquire), total);

        let mut vals = Arc::try_unwrap(popped_vals).unwrap().into_inner().unwrap();
        vals.sort_unstable();
        assert_eq!(vals.len(), total);
        for (i, v) in vals.into_iter().enumerate() {
            assert_eq!(v, i);
        }

        let ptrs = Arc::try_unwrap(popped_ptrs).unwrap().into_inner().unwrap();
        for p in ptrs {
            unsafe { drop_node(p) };
        }

        assert!(unsafe { s.pop() }.is_none());
    }
}
