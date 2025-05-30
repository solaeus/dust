use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::Span;

use super::{Item, Path};

#[derive(Clone, Debug, Default, PartialEq, Serialize)]
pub struct Module<'a> {
    pub items: HashMap<Path<'a>, (Item<'a>, Span)>,
}

impl<'a> Module<'a> {
    pub fn new() -> Self {
        Module {
            items: HashMap::new(),
        }
    }

    pub fn get_item(&self, path: &Path<'a>) -> Option<&(Item<'a>, Span)> {
        let mut current_module = self;

        for module_name in path.module_names() {
            if let Some((item, _)) = current_module.items.get(&module_name) {
                if let Item::Module(module) = item {
                    current_module = &module;

                    continue;
                } else {
                    return None; // Path points to a non-module item
                }
            }

            return None; // Module not found
        }

        current_module.items.get(&path.item_name())
    }
}

impl<'a, 'de: 'a> Deserialize<'de> for Module<'a> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let items = HashMap::<Path<'a>, (Item<'a>, Span)>::deserialize(deserializer)?;

        Ok(Module { items })
    }
}
