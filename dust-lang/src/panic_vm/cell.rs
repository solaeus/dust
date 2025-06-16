use std::sync::{Arc, RwLock};

use crate::{DustString, FullChunk, List};

#[derive(Debug)]
pub struct Cell<C> {
    pub value: Arc<RwLock<CellValue<C>>>,
}

impl<C> Cell<C> {
    pub fn empty() -> Self {
        Self {
            value: Arc::new(RwLock::new(CellValue::Empty)),
        }
    }

    pub fn set_boolean(&self, boolean: bool) {
        *self.value.write().expect("Failed to write cell") = CellValue::Boolean(boolean);
    }

    pub fn set_byte(&self, byte: u8) {
        *self.value.write().expect("Failed to write cell") = CellValue::Byte(byte);
    }

    pub fn set_character(&self, character: char) {
        *self.value.write().expect("Failed to write cell") = CellValue::Character(character);
    }

    pub fn set_float(&self, float: f64) {
        *self.value.write().expect("Failed to write cell") = CellValue::Float(float);
    }

    pub fn set_integer(&self, integer: i64) {
        *self.value.write().expect("Failed to write cell") = CellValue::Integer(integer);
    }

    pub fn set_string(&self, string: DustString) {
        *self.value.write().expect("Failed to write cell") = CellValue::String(string);
    }

    pub fn set_list(&self, list: List<C>) {
        *self.value.write().expect("Failed to write cell") = CellValue::List(list);
    }

    pub fn set_function(&self, function: Arc<FullChunk>) {
        *self.value.write().expect("Failed to write cell") = CellValue::Function(function);
    }
}

#[derive(Debug)]
pub enum CellValue<C> {
    Empty,
    Boolean(bool),
    Byte(u8),
    Character(char),
    Float(f64),
    Integer(i64),
    String(DustString),
    List(List<C>),
    Function(Arc<FullChunk>),
}
