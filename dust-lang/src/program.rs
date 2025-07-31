use serde::{Deserialize, Serialize};

use crate::Chunk;

#[derive(Debug, Clone, Eq, PartialEq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Program {
    pub main_chunk: Chunk,
    pub cell_count: u16,
    pub prototypes: Vec<Chunk>,
}
