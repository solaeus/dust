use std::sync::Arc;

use crate::{Chunk, DustString, List};

const STACK_SIZE: usize = 64;

#[derive(Debug)]
pub struct Memory<C> {
    pub stack: [Register; STACK_SIZE],
    pub top: usize,

    pub heap: Vec<Register>,

    pub objects: Vec<Object<C>>,
}

impl<C: Chunk> Memory<C> {
    pub fn new() -> Self {
        Memory {
            stack: [Register(0); STACK_SIZE],
            top: 0,
            heap: Vec::with_capacity(0),
            objects: Vec::with_capacity(0),
        }
    }

    pub fn allocate_registers(&mut self, count: usize) {
        if self.top + count <= STACK_SIZE {
            self.top += count;
        } else {
            let additional = count - STACK_SIZE;

            self.heap.reserve_exact(additional);
            self.heap.resize(self.heap.len() + additional, Register(0));
        }
    }

    pub fn store_object(&mut self, object: Object<C>) -> Register {
        let object_index = self.objects.len();
        let register = Register::index(object_index);

        self.objects.push(object);

        register
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.top + self.heap.len()
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.top == 0 && self.heap.is_empty()
    }

    #[inline]
    pub fn push(&mut self, value: Register) {
        if self.top < self.stack.len() {
            self.stack[self.top] = value;
            self.top += 1;
        } else {
            self.heap.push(value);
        }
    }

    #[inline]
    pub fn pop(&mut self) -> Option<Register> {
        if self.top > 0 {
            self.top -= 1;
            Some(self.stack[self.top])
        } else if !self.heap.is_empty() {
            Some(self.heap.pop().unwrap())
        } else {
            None
        }
    }

    #[inline]
    pub fn get(&self, index: usize) -> Option<Register> {
        if index < self.top {
            Some(self.stack[index])
        } else if index < self.top + self.heap.len() {
            Some(self.heap[index - self.top])
        } else {
            None
        }
    }

    #[inline]
    pub fn get_mut(&mut self, index: usize) -> Option<&mut Register> {
        if index < self.top {
            Some(&mut self.stack[index])
        } else if index < self.top + self.heap.len() {
            Some(&mut self.heap[index - self.top])
        } else {
            None
        }
    }

    #[inline]
    pub fn set(&mut self, index: usize, value: Register) {
        if index < self.top {
            self.stack[index] = value;
        } else if index < self.top + self.heap.len() {
            self.heap[index - self.top] = value;
        }
    }
}

impl<C: Chunk> Default for Memory<C> {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug)]
pub enum Object<C> {
    Empty,
    Function(Arc<C>),
    ValueList(List<C>),
    RegisterList(Vec<Register>),
    String(DustString),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Register(u64);

impl Register {
    pub fn boolean(boolean: bool) -> Self {
        Register(boolean as u64)
    }

    pub fn as_boolean(&self) -> bool {
        self.0 != 0
    }

    pub fn byte(byte: u8) -> Self {
        Register(byte as u64)
    }

    pub fn as_byte(&self) -> u8 {
        self.0 as u8
    }

    pub fn character(character: char) -> Self {
        Register(character as u64)
    }

    pub fn as_character(&self) -> char {
        char::from_u32(self.0 as u32).unwrap_or_default()
    }

    pub fn float(float: f64) -> Self {
        Register(float.to_bits())
    }

    pub fn as_float(&self) -> f64 {
        f64::from_bits(self.0)
    }

    pub fn integer(integer: i64) -> Self {
        let bytes = integer.to_le_bytes();

        Register(u64::from_le_bytes(bytes))
    }

    pub fn as_integer(&self) -> i64 {
        let bytes = self.0.to_le_bytes();

        i64::from_le_bytes(bytes)
    }

    pub fn index(index: usize) -> Self {
        Register(index as u64)
    }

    pub fn as_index(&self) -> usize {
        self.0 as usize
    }
}
