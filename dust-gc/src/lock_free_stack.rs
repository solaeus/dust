use std::sync::atomic::Ordering;

use crossbeam_epoch::{Atomic, Owned, pin};

pub struct LockFreeStack<T: Sized> {
    head: Atomic<Node<T>>,
}

impl<T: Sized + Copy> LockFreeStack<T> {
    pub fn new() -> Self {
        Self {
            head: Atomic::null(),
        }
    }

    pub fn push(&self, payload: T) {
        let mut node = Owned::new(Node {
            next: Atomic::null(),
            payload,
        });
        let guard = pin();

        loop {
            let head = self.head.load(Ordering::Acquire, &guard);

            node.next.store(head, Ordering::Relaxed);

            match self.head.compare_exchange(
                head,
                node,
                Ordering::Relaxed,
                Ordering::Relaxed,
                &guard,
            ) {
                Ok(_) => return,
                Err(error) => node = error.new,
            }
        }
    }

    pub fn pop(&self) -> Option<T> {
        let guard = pin();

        loop {
            let head = self.head.load(Ordering::Acquire, &guard);

            if head.is_null() {
                return None;
            }

            let next = unsafe { head.deref().next.load(Ordering::Relaxed, &guard) };

            if self
                .head
                .compare_exchange(head, next, Ordering::AcqRel, Ordering::Acquire, &guard)
                .is_ok()
            {
                let payload = unsafe { head.into_owned().payload };

                return Some(payload);
            }
        }
    }
}

struct Node<T> {
    next: Atomic<Node<T>>,
    payload: T,
}

#[cfg(test)]
mod tests {
    use std::{sync::Arc, thread::spawn};

    use super::*;

    #[test]
    fn push_and_pop() {
        let lock_free_stack = Arc::new(LockFreeStack::new());

        let stack_0 = lock_free_stack.clone();
        let stack_1 = lock_free_stack.clone();
        let stack_2 = lock_free_stack.clone();

        let thread_handles = [
            spawn(move || {
                stack_0.push(0);
            }),
            spawn(move || {
                stack_1.push(1);
            }),
            spawn(move || {
                stack_2.push(2);
            }),
        ];

        for thread in thread_handles {
            thread.join().unwrap();
        }

        let popped = [
            lock_free_stack.pop().unwrap(),
            lock_free_stack.pop().unwrap(),
            lock_free_stack.pop().unwrap(),
        ];

        assert!(popped.contains(&0));
        assert!(popped.contains(&1));
        assert!(popped.contains(&2));
    }
}
