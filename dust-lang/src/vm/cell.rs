use std::sync::{Arc, RwLock};

use crate::{DustString, List};

#[derive(Debug, Default)]
pub struct Cell<C> {
    pub lock: Arc<RwLock<CellValue<C>>>,
}

impl<C> Cell<C> {
    pub fn empty() -> Self {
        Self {
            lock: Arc::new(RwLock::new(CellValue::Empty)),
        }
    }
}

#[derive(Debug, Default)]
pub enum CellValue<C> {
    #[default]
    Empty,

    Boolean(bool),
    Byte(u8),
    Character(char),
    Float(f64),
    Integer(i64),
    String(DustString),
    List(List<C>),
    Function(Arc<C>),
}
