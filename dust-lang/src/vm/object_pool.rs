use std::collections::HashSet;

use slab::Slab;

use crate::{List, Object, vm::object::ObjectData};

pub struct ObjectPool {
    objects: Slab<Object>,
    roots: HashSet<usize>,
}

impl ObjectPool {
    pub fn new() -> Self {
        Self {
            objects: Slab::new(),
            roots: HashSet::new(),
        }
    }

    pub fn allocate(&mut self, object: Object) -> usize {
        let key = self.objects.insert(object);
        self.roots.insert(key);
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

    /// Mark-and-sweep GC: mark from roots, then sweep.
    pub fn collect_garbage(&mut self) {
        for root in self.roots.clone() {
            self.mark(root);
        }

        self.sweep();
    }

    fn mark(&mut self, key: usize) {
        if let Some(object) = self.objects.get_mut(key) {
            if object.mark {
                return;
            }

            object.mark = true;

            if let ObjectData::ValueList(list) = &object.data {
                for child_key in list_object_keys(list) {
                    self.mark(child_key);
                }
            }
        }
    }

    fn sweep(&mut self) {
        self.objects.retain(|key, object| {
            let keep = object.mark;

            if !object.mark {
                self.roots.remove(&key);
            }

            object.mark = false;

            keep
        });
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
