use std::{pin::Pin, ptr};

use crate::{Object, OperandType, Register};

pub struct ObjectPool {
    objects: Vec<Pin<Box<Object>>>,
    objects_heap_size: usize,
}

impl ObjectPool {
    pub fn new() -> Self {
        Self {
            objects: Vec::new(),
            objects_heap_size: 0,
        }
    }

    pub fn pool_size(&self) -> usize {
        let pointer_buffer_size = self.objects.capacity() * size_of::<Pin<Box<Object>>>();

        pointer_buffer_size + self.objects_heap_size
    }

    pub fn allocate(&mut self, object: Object) -> *mut Object {
        self.objects_heap_size += object.size();

        let mut pinned = Box::pin(object);
        let pointer = &mut *pinned as *mut Object;

        self.objects.push(pinned);

        pointer
    }

    pub fn mark(&mut self, key: usize) {
        if let Some(object) = self.objects.get_mut(key) {
            object.mark = true;
        }
    }

    pub fn get(&self, key: usize) -> Option<&Object> {
        self.objects.get(key).map(|object| &**object)
    }

    pub fn get_mut(&mut self, key: usize) -> Option<&mut Object> {
        self.objects.get_mut(key).map(|object| &mut **object)
    }

    fn _collect_garbage(&mut self, registers: &[Register], register_tags: &[OperandType]) {
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
                let pointer = unsafe { register.object_pointer };

                for object in &mut self.objects {
                    if ptr::eq(&**object, pointer) {
                        object.mark = true;
                        break;
                    }
                }
            }
        }

        let mut i = 0;
        self.objects_heap_size = 0;

        while i < self.objects.len() {
            if self.objects[i].mark {
                self.objects_heap_size += self.objects[i].size();
                self.objects[i].mark = false;

                i += 1;
            } else {
                self.objects.remove(i);
            }
        }
    }
}

impl Default for ObjectPool {
    fn default() -> Self {
        Self::new()
    }
}
