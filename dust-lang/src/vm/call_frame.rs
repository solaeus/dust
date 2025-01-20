use std::{
    fmt::{self, Debug, Display, Formatter},
    sync::Arc,
};

use crate::{Chunk, DustString};

use super::register_table::RegisterTable;

#[derive(Debug)]
pub struct CallFrame {
    pub prototype: Arc<Chunk>,
    pub registers: RegisterTable,
    pub return_register: u16,
    pub instruction_pointer: usize,
}

impl CallFrame {
    pub fn new(prototype: Arc<Chunk>, return_register: u16) -> Self {
        Self {
            prototype,
            return_register,
            instruction_pointer: 0,
            registers: RegisterTable::new(),
        }
    }
}

impl Display for CallFrame {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(
            f,
            "{} IP = {}",
            self.prototype
                .name
                .as_ref()
                .unwrap_or(&DustString::from("anonymous")),
            self.instruction_pointer,
        )
    }
}
