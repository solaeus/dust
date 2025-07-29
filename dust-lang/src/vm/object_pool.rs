use std::collections::{HashSet, VecDeque};

use slab::Slab;

use crate::{List, Object, vm::object::ObjectValue};

pub struct ObjectPool {
    objects: Slab<Object>,
    roots: HashSet<usize>,
    allocations_since_collection: usize,
}

impl ObjectPool {
    const MAX_ALLOCATIONS_BEFORE_COLLECTION: usize = {
        if cfg!(debug_assertions) {
            10_000
        } else {
            100_000
        }
    };

    pub fn new() -> Self {
        Self {
            objects: Slab::new(),
            roots: HashSet::new(),
            allocations_since_collection: 0,
        }
    }

    pub fn allocate(&mut self, object: Object) -> usize {
        let key = self.objects.insert(object);

        self.roots.insert(key);

        self.allocations_since_collection += 1;

        key
    }

    /// Remove an object from the root set when you know it's no longer needed.
    pub fn unroot(&mut self, key: usize) {
        self.roots.remove(&key);
    }

    /// Get a reference to an object by key.
    pub fn get(&self, key: usize) -> Option<&Object> {
        self.objects.get(key)
    }

    /// Get a mutable reference to an object by key.
    pub fn get_mut(&mut self, key: usize) -> Option<&mut Object> {
        self.objects.get_mut(key)
    }

    pub fn collect_garbage(&mut self) {
        if self.allocations_since_collection < Self::MAX_ALLOCATIONS_BEFORE_COLLECTION {
            return;
        }

        let mut worklist: VecDeque<usize> = self.roots.iter().copied().collect();

        while let Some(key) = worklist.pop_front() {
            if let Some(object) = self.objects.get_mut(key) {
                if object.mark {
                    continue;
                }

                object.mark = true;

                if let ObjectValue::List(list) = &object.value {
                    for child_key in list_object_keys(list) {
                        worklist.push_back(child_key);
                    }
                }
            }
        }

        self.objects.retain(|key, obj| {
            let keep = obj.mark;

            if !keep {
                self.roots.remove(&key);
            }

            obj.mark = false;

            keep
        });

        self.allocations_since_collection = 0;
    }
}

impl Default for ObjectPool {
    fn default() -> Self {
        Self::new()
    }
}

fn list_object_keys(_list: &List) -> Vec<usize> {
    todo!()
}
