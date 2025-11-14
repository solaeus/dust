use std::sync::{Arc, RwLock};

use crate::{prototype::Prototype, value::List};

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
    String(String),
    List(List),
    Function(Arc<Prototype>),
}
