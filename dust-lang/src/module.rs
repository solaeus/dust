use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::{Item, Span};

use super::Path;

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Module {
    pub items: HashMap<Path, (Item, Span)>,
}

impl Module {
    pub fn new() -> Self {
        Module {
            items: HashMap::new(),
        }
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Module {
            items: HashMap::with_capacity(capacity),
        }
    }

    pub fn find_item<'b>(&'b self, item_path: &'b Path) -> Option<&'b (Item, Span)> {
        let mut current_module = self;

        for module_name in item_path.modules() {
            if let Some((Item::Module(module), _)) = current_module.items.get(&module_name) {
                current_module = module;
            } else {
                return None;
            }
        }

        let item_name = item_path.item();

        current_module.items.get(&item_name)
    }
}
