use std::time::Instant;

use bumpalo::{Bump, collections::Vec as BumpVec};
use tracing::{debug, trace};

use crate::jit_vm::{Object, Register, RegisterTag, object::ObjectValue};

#[repr(C)]
pub struct ObjectPool<'a> {
    object_slots: BumpVec<'a, Option<Object>>,
    generations: Vec<u32>,
    free_slots: Vec<u32>,
    arena: &'a Bump,

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

impl<'a> ObjectPool<'a> {
    pub fn new(arena: &'a Bump, minimum_sweep_threshold: usize, minimum_heap_size: usize) -> Self {
        Self {
            object_slots: BumpVec::new_in(arena),
            generations: Vec::new(),
            free_slots: Vec::new(),
            arena,
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
    ) -> ObjectIndex {
        if self.allocated >= self.next_sweep_threshold {
            let length = self.object_slots.len();
            let allocated = self.allocated;
            let start = Instant::now();

            self.mark(registers, register_tags);
            self.sweep();

            let collected = length - self.object_slots.len();
            let deallocated = allocated - self.allocated;
            let elapsed = start.elapsed().as_nanos();
            self.next_sweep_threshold =
                (self.allocated + self.minimum_sweep_threshold).max(self.minimum_heap_size);

            debug!("Collected {collected} objects, deallocated {deallocated} bytes in {elapsed}ns");

            self.total_objects_deallocated += collected;
            self.total_bytes_deallocated += deallocated;
            self.total_collection_time += elapsed;
            self.total_collections += 1;
        }

        let size = object.size();
        self.allocated += size;
        self.total_bytes_allocated += size;
        self.total_objects_allocated += 1;

        trace!("Allocating object with {size} bytes: {object:?}");

        if let Some(free_index) = self.free_slots.pop() {
            self.object_slots[free_index as usize] = Some(object);

            ObjectIndex {
                slot_index: free_index,
                generation: self.generations[free_index as usize],
            }
        } else {
            let slot_index = self.object_slots.len();

            self.object_slots.push(Some(object));
            self.generations.push(0);

            ObjectIndex {
                slot_index: slot_index as u32,
                generation: 0,
            }
        }
    }

    pub fn get(&self, index: ObjectIndex) -> Option<&Object> {
        let slot_index = index.slot_index as usize;

        if slot_index >= self.object_slots.len() || self.generations[slot_index] != index.generation
        {
            return None;
        }

        self.object_slots[slot_index].as_ref()
    }

    pub fn get_mut(&mut self, index: ObjectIndex) -> Option<&mut Object> {
        let slot_index = index.slot_index as usize;

        if slot_index >= self.object_slots.len() || self.generations[slot_index] != index.generation
        {
            return None;
        }

        self.object_slots[slot_index].as_mut()
    }

    pub fn take(&mut self, obj_index: ObjectIndex) -> Option<Object> {
        let index = obj_index.slot_index as usize;

        if index >= self.object_slots.len() {
            return None;
        }

        if self.generations[index] != obj_index.generation {
            return None;
        }

        let object = self.object_slots[index].take()?;
        let size = object.size();

        self.allocated -= size;
        self.total_bytes_deallocated += size;
        self.total_objects_deallocated += 1;

        self.free_slots.push(index as u32);
        self.generations[index] = self.generations[index].wrapping_add(1);

        Some(object)
    }

    fn sweep(&mut self) {
        self.allocated = 0;

        for (index, slot) in self.object_slots.iter_mut().enumerate() {
            if let Some(object) = slot {
                if object.mark {
                    self.allocated += object.size();
                    object.mark = false;
                } else {
                    *slot = None;

                    self.free_slots.push(index as u32);

                    self.generations[index] = self.generations[index].wrapping_add(1);
                }
            }
        }
    }

    fn mark(&mut self, registers: &[Register], register_tags: &[RegisterTag]) {
        let mut worklist: Vec<u32> = Vec::new();

        for (register, tag) in registers.iter().zip(register_tags.iter()) {
            if *tag == RegisterTag::OBJECT {
                let slot_index = ObjectIndex::decode(unsafe { register.object_index }).slot_index;

                worklist.push(slot_index);
            }
        }

        while let Some(slot_index) = worklist.pop() {
            let object = match self.object_slots.get_mut(slot_index as usize) {
                Some(Some(object)) => object,
                _ => continue,
            };

            if object.mark {
                continue;
            }

            object.mark = true;

            if let ObjectValue::ObjectList(children) = &object.value {
                for child_index in children {
                    let slot_index = ObjectIndex::decode(*child_index).slot_index;

                    worklist.push(slot_index);
                }
            }
        }
    }

    pub fn report(&self) -> String {
        const INDENT: &str = "    ";

        format!(
            "Object Pool Report:\n\
             {INDENT}- Current Allocations:\n\
             {INDENT}{INDENT}- {} objects\n\
             {INDENT}{INDENT}- {} bytes\n\
             {INDENT}- Runtime Allocations:\n\
             {INDENT}{INDENT}- {} objects\n\
             {INDENT}{INDENT}- {} bytes\n\
             {INDENT}- Runtime Deallocations:\n\
             {INDENT}{INDENT}- {} objects\n\
             {INDENT}{INDENT}- {} bytes\n\
             {INDENT}Spent {}ms on {} collections",
            self.object_slots.len(),
            self.allocated,
            self.total_objects_allocated,
            self.total_bytes_allocated,
            self.total_objects_deallocated,
            self.total_bytes_deallocated,
            self.total_collection_time / 1_000,
            self.total_collections
        )
    }
}

#[derive(Clone, Copy, Debug)]
pub struct ObjectIndex {
    pub slot_index: u32,
    pub generation: u32,
}

impl ObjectIndex {
    pub fn encode(&self) -> u64 {
        ((self.generation as u64) << 32) | (self.slot_index as u64)
    }

    pub fn decode(encoded: u64) -> Self {
        Self {
            slot_index: encoded as u32,
            generation: (encoded >> 32) as u32,
        }
    }
}
