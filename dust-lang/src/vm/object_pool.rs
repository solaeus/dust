use slab::Slab;

use crate::{Object, OperandType, Register};

pub struct ObjectPool {
    objects: Slab<Object>,
    objects_heap_size: usize,
}

impl ObjectPool {
    const MAX_SIZE: usize = {
        if cfg!(debug_assertions) {
            1_000
        } else {
            4_000_000_000
        }
    };

    pub fn new() -> Self {
        Self {
            objects: Slab::new(),
            objects_heap_size: 0,
        }
    }

    pub fn pool_size(&self) -> usize {
        let slab_buffer_size = self.objects.capacity() * size_of::<Object>();

        slab_buffer_size + self.objects_heap_size
    }

    pub fn allocate(&mut self, object: Object) -> usize {
        self.objects_heap_size += object.size();

        self.objects.insert(object)
    }

    pub fn mark(&mut self, key: usize) {
        if let Some(object) = self.objects.get_mut(key) {
            object.mark = true;
        }
    }

    pub fn get(&self, key: usize) -> Option<&Object> {
        self.objects.get(key)
    }

    pub fn get_mut(&mut self, key: usize) -> Option<&mut Object> {
        self.objects.get_mut(key)
    }

    fn collect_garbage(&mut self, registers: &[Register], register_tags: &[OperandType]) {
        for (index, tag) in register_tags.iter().enumerate() {
            if matches!(
                *tag,
                OperandType::STRING
                    | OperandType::LIST
                    | OperandType::LIST_BOOLEAN
                    | OperandType::LIST_BYTE
                    | OperandType::LIST_CHARACTER
                    | OperandType::LIST_FLOAT
                    | OperandType::LIST_INTEGER
                    | OperandType::LIST_STRING
                    | OperandType::LIST_LIST
                    | OperandType::LIST_FUNCTION
            ) {
                let register = &registers[index];
                let key = unsafe { register.object_key };

                if let Some(object) = self.objects.get_mut(key) {
                    object.mark = true;
                }
            }
        }

        self.objects_heap_size = 0;

        self.objects.retain(|_, object| {
            let keep = object.mark;

            if keep {
                self.objects_heap_size += object.size();
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
