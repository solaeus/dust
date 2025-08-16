use std::pin::Pin;

use tracing::trace;

use crate::Object;

const MINIMUM_SWEEP_THRESHOLD: usize = if cfg!(debug_assertions) {
    1
} else {
    1024 * 1024
};
const DEFAULT_SWEEP_THRESHOLD: usize = if cfg!(debug_assertions) {
    4
} else {
    1024 * 1024 * 4
};

#[repr(C)]
pub struct ObjectPool {
    objects: Vec<Pin<Box<Object>>>,
    pub allocated: usize,
    pub next_sweep_threshold: usize,
}

impl ObjectPool {
    pub fn new() -> Self {
        Self {
            objects: Vec::new(),
            allocated: 0,
            next_sweep_threshold: DEFAULT_SWEEP_THRESHOLD,
        }
    }

    pub fn allocate(&mut self, object: Object) -> *mut Object {
        let size = object.size();
        self.allocated += size;

        trace!("Allocating object with {} bytes", size);

        if self.allocated >= self.next_sweep_threshold {
            trace!(
                "Sweeping object pool: {} objects, {} bytes",
                self.objects.len(),
                self.allocated
            );

            self.sweep();

            self.next_sweep_threshold =
                (self.allocated + MINIMUM_SWEEP_THRESHOLD).max(DEFAULT_SWEEP_THRESHOLD);

            trace!(
                "Swept object pool: {} retained objects, {} bytes",
                self.objects.len(),
                self.allocated
            );
        }

        let mut pinned = Box::pin(object);
        let pointer = &mut *pinned as *mut Object;

        self.objects.push(pinned);

        pointer
    }

    pub fn get(&self, key: usize) -> Option<&Object> {
        self.objects.get(key).map(|object| &**object)
    }

    pub fn get_mut(&mut self, key: usize) -> Option<&mut Object> {
        self.objects.get_mut(key).map(|object| &mut **object)
    }

    fn sweep(&mut self) {
        self.allocated = 0;

        self.objects.retain_mut(|object| {
            let keep = object.mark;

            if keep {
                self.allocated += object.size();
                object.mark = false;
            }

            keep
        });
    }
}

impl Default for ObjectPool {
    fn default() -> Self {
        Self::new()
    }
}
