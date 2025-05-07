use std::collections::HashMap;

use crate::ConcreteValue;

use super::path::Path;

#[derive(Debug, Clone)]
pub struct Module {
    items: HashMap<Path, ModuleItem>,
}

impl Module {
    pub fn new() -> Self {
        Module {
            items: HashMap::new(),
        }
    }
}

#[derive(Debug, Clone)]
pub enum ModuleItem {
    Constant(ConcreteValue),
    Module(Module),
}
