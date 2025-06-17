use std::sync::Arc;

use hashbrown::HashSet;

use crate::{Address, Chunk, DEFAULT_REGISTER_COUNT, DustString, List, instruction::OperandType};

#[derive(Debug)]
pub struct Memory<C> {
    pub heap: Heap<C>,

    pub closed: HashSet<(Address, OperandType)>,

    pub stack: Stack<DEFAULT_REGISTER_COUNT>,
}

impl<'a, C: Chunk<'a>> Memory<C> {
    pub fn new(chunk: &C) -> Self {
        Memory {
            heap: Heap::new(chunk),
            closed: HashSet::new(),
            stack: Stack::new(),
        }
    }
}

#[derive(Debug)]
pub struct Heap<C> {
    pub booleans: Vec<HeapSlot<bool>>,
    pub bytes: Vec<HeapSlot<u8>>,
    pub characters: Vec<HeapSlot<char>>,
    pub floats: Vec<HeapSlot<f64>>,
    pub integers: Vec<HeapSlot<i64>>,
    pub strings: Vec<HeapSlot<DustString>>,
    pub lists: Vec<HeapSlot<List<C>>>,
    pub functions: Vec<HeapSlot<Arc<C>>>,
}

impl<'a, C: Chunk<'a>> Heap<C> {
    pub fn new(chunk: &C) -> Self {
        Self {
            booleans: vec![HeapSlot::Closed; chunk.boolean_memory_length() as usize],
            bytes: vec![HeapSlot::Closed; chunk.byte_memory_length() as usize],
            characters: vec![HeapSlot::Closed; chunk.character_memory_length() as usize],
            floats: vec![HeapSlot::Closed; chunk.float_memory_length() as usize],
            integers: vec![HeapSlot::Closed; chunk.integer_memory_length() as usize],
            strings: vec![HeapSlot::Closed; chunk.string_memory_length() as usize],
            lists: vec![HeapSlot::Closed; chunk.list_memory_length() as usize],
            functions: vec![HeapSlot::Closed; chunk.function_memory_length() as usize],
        }
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub enum HeapSlot<T> {
    #[default]
    Closed,
    Open(T),
}

#[derive(Debug)]
pub struct Stack<const LENGTH: usize> {
    pub booleans: [bool; LENGTH],
    pub bytes: [u8; LENGTH],
    pub characters: [char; LENGTH],
    pub floats: [f64; LENGTH],
    pub integers: [i64; LENGTH],
}

impl<const LENGTH: usize> Stack<LENGTH> {
    pub fn new() -> Self {
        Stack {
            booleans: [false; LENGTH],
            bytes: [0; LENGTH],
            characters: [char::default(); LENGTH],
            floats: [0.0; LENGTH],
            integers: [0; LENGTH],
        }
    }
}

impl<const LENGTH: usize> Default for Stack<LENGTH> {
    fn default() -> Self {
        Self::new()
    }
}
