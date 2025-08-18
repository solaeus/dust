use std::{pin::Pin, time::Instant};

use tracing::{debug, trace};

use crate::{
    Object, Register,
    jit_vm::{RegisterTag, object::ObjectValue},
};

#[repr(C)]
pub struct ObjectPool {
    objects: Vec<Pin<Box<Object>>>,

    allocated: usize,
    next_sweep_threshold: usize,

    minimum_sweep_threshold: usize,
    minimum_heap_size: usize,

    total_objects_allocated: usize,
    total_bytes_allocated: usize,

    total_objects_deallocated: usize,
    total_bytes_deallocated: usize,

    total_collection_time: u128,
    total_collections: usize,
}

impl ObjectPool {
    pub fn new(minimum_sweep_threshold: usize, minimum_heap_size: usize) -> Self {
        Self {
            objects: Vec::new(),
            allocated: 0,
            next_sweep_threshold: minimum_heap_size,
            minimum_sweep_threshold,
            minimum_heap_size,
            total_objects_allocated: 0,
            total_bytes_allocated: 0,
            total_objects_deallocated: 0,
            total_bytes_deallocated: 0,
            total_collection_time: 0,
            total_collections: 0,
        }
    }

    pub fn allocate(
        &mut self,
        object: Object,
        registers: &[Register],
        register_tags: &[RegisterTag],
    ) -> *mut Object {
        if self.allocated >= self.next_sweep_threshold {
            let length = self.objects.len();
            let allocated = self.allocated;
            let start = Instant::now();

            Self::mark(registers, register_tags);
            self.sweep();

            let collected = length - self.objects.len();
            let deallocated = allocated - self.allocated;
            let elapsed = start.elapsed().as_nanos();
            self.next_sweep_threshold =
                (self.allocated + self.minimum_sweep_threshold).max(self.minimum_heap_size);

            debug!(
                "Collected {collected} objects, deallocated {deallocated} bytes in {}ns",
                elapsed
            );

            self.total_objects_deallocated += collected;
            self.total_bytes_deallocated += deallocated;
            self.total_collection_time += elapsed;
            self.total_collections += 1;
        }

        let size = object.size();
        self.allocated += size;
        self.total_bytes_allocated += size;
        self.total_objects_allocated += 1;

        trace!("Allocating object with {size} bytes: {object}");

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

    fn mark(registers: &[Register], register_tags: &[RegisterTag]) {
        for (register, tag) in registers.iter().zip(register_tags.iter()) {
            if *tag == RegisterTag::OBJECT {
                let object = unsafe { &mut *register.object_pointer };

                Self::mark_object(object);
            }
        }
    }

    fn mark_object(object: &mut Object) {
        object.mark = true;

        if let ObjectValue::ObjectList(object_pointers) = &object.value {
            for object_pointer in object_pointers {
                let object = unsafe { &mut **object_pointer };

                Self::mark_object(object);
            }
        }
    }

    pub fn report(&self) -> String {
        format!(
            "ObjectPool Report:\n\
             Allocated: {} bytes\n\
             Total Bytes Deallocated: {}\n\
             Total Objects Allocated: {}\n\
             Total Bytes Allocated: {}\n\
             Total Collection Time: {} ms\n\
             Total Collections: {}",
            self.allocated,
            self.total_bytes_deallocated,
            self.total_objects_allocated,
            self.total_bytes_allocated,
            self.total_collection_time / 1_000,
            self.total_collections
        )
    }
}
