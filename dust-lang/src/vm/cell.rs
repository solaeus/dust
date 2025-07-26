use std::sync::{Arc, RwLock};

use crate::{Chunk, DustString, List};

#[derive(Debug, Default)]
pub struct Cell {
    pub lock: Arc<RwLock<CellValue>>,
}

impl Cell {
    pub fn empty() -> Self {
        Self {
            lock: Arc::new(RwLock::new(CellValue::Empty)),
        }
    }
}

#[derive(Debug, Default)]
pub enum CellValue {
    #[default]
    Empty,

    Boolean(bool),
    Byte(u8),
    Character(char),
    Float(f64),
    Integer(i64),
    String(DustString),
    List(List),
    Function(Arc<Chunk>),
}
