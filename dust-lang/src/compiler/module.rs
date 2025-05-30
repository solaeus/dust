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

    pub fn get_item<'b>(&'b self, path: &'b Path<'a>) -> Option<&'b (Item<'a>, Span)> {
        let mut current_module = self;

        for module_name in path.module_names() {
            if let Some((item, _)) = current_module
                .items
                .get(&Path::new_borrowed(module_name).unwrap())
            {
                if let Item::Module(module) = item {
                    current_module = module;

                    continue;
                } else {
                    return None; // Path points to a non-module item
                }
            }

            return None; // Module not found
        }

        current_module
            .items
            .get(&Path::new_borrowed(path.item_name()).unwrap())
    }
}

impl<'a, 'de: 'a> Deserialize<'de> for Module<'a> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct ModuleVisitor<'a> {
            marker: std::marker::PhantomData<&'a ()>,
        }

        impl<'a> serde::de::Visitor<'a> for ModuleVisitor<'a> {
            type Value = Module<'a>;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("a module with items")
            }

            fn visit_map<M>(self, mut map: M) -> Result<Self::Value, M::Error>
            where
                M: serde::de::MapAccess<'a>,
            {
                let mut items = HashMap::new();

                while let Some((key, value)) = map.next_entry::<Path<'a>, (Item<'a>, Span)>()? {
                    items.insert(key, value);
                }

                Ok(Module { items })
            }
        }

        deserializer.deserialize_struct(
            "Module",
            &["items"],
            ModuleVisitor {
                marker: std::marker::PhantomData,
            },
        )
    }
}
