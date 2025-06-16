use std::sync::Arc;

use hashbrown::HashSet;

use crate::{Address, Chunk, DEFAULT_REGISTER_COUNT, DustString, FullChunk, List};

#[derive(Debug)]
pub struct Memory<C> {
    pub heap: Heap<C>,

    pub closed: HashSet<Address>,

    pub stack: Stack<DEFAULT_REGISTER_COUNT>,
}

impl<C: Chunk> Memory<C> {
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
    pub booleans: Vec<bool>,
    pub bytes: Vec<u8>,
    pub characters: Vec<char>,
    pub floats: Vec<f64>,
    pub integers: Vec<i64>,
    pub strings: Vec<DustString>,
    pub lists: Vec<List<C>>,
    pub functions: Vec<Arc<FullChunk>>,
}

impl<C: Chunk> Heap<C> {
    #[expect(clippy::rc_clone_in_vec_init)]
    pub fn new(chunk: &C) -> Self {
        Self {
            booleans: vec![false; chunk.boolean_memory_length() as usize],
            bytes: vec![0; chunk.byte_memory_length() as usize],
            characters: vec![char::default(); chunk.character_memory_length() as usize],
            floats: vec![0.0; chunk.float_memory_length() as usize],
            integers: vec![0; chunk.integer_memory_length() as usize],
            strings: vec![DustString::new(); chunk.string_memory_length() as usize],
            lists: vec![List::boolean([]); chunk.list_memory_length() as usize],
            functions: vec![
                Arc::new(FullChunk::default());
                chunk.function_memory_length() as usize
            ],
        }
    }
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
