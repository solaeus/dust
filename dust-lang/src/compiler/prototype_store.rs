use std::collections::HashMap;

use crate::{Chunk, Path};

#[derive(Debug)]
pub struct PrototypeStore {
    chunks: Vec<Chunk>,
    index_map: HashMap<Path, usize>,
}

impl PrototypeStore {
    pub fn new() -> Self {
        Self {
            chunks: Vec::new(),
            index_map: HashMap::new(),
        }
    }

    pub fn len(&self) -> usize {
        self.chunks.len()
    }

    pub fn insert(&mut self, chunk: Chunk) -> usize {
        let index = self.chunks.len();

        if let Some(name) = &chunk.name {
            self.index_map.insert(name.clone(), index);
        }

        self.chunks.push(chunk);

        index
    }

    pub fn get_by_name(&self, name: &str) -> Option<&Chunk> {
        self.index_map
            .get(name)
            .and_then(|&index| self.chunks.get(index))
    }

    pub fn _get_by_index(&self, index: usize) -> Option<&Chunk> {
        self.chunks.get(index)
    }

    pub fn pop(&mut self) -> Option<Chunk> {
        if let Some(chunk) = self.chunks.pop() {
            if let Some(name) = &chunk.name {
                self.index_map.remove(name);
            }

            Some(chunk)
        } else {
            None
        }
    }

    pub fn into_chunks(self) -> Vec<Chunk> {
        self.chunks
    }
}
