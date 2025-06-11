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

    pub fn with_capacity(capacity: usize) -> Self {
        Module {
            items: HashMap::with_capacity(capacity),
        }
    }

    pub fn get_item(&self, path: &Path<'a>) -> Option<&(Item<'a>, Span)> {
        if let Some(found) = self.items.get(path) {
            return Some(found);
        }

        for module in self.items.iter().filter_map(|(_, (item, _))| {
            if let Item::Module(module) = item {
                Some(module)
            } else {
                None
            }
        }) {
            if let Some(found) = module.get_item(path) {
                return Some(found);
            }
        }

        None
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

#[macro_export]
macro_rules! find_item {
    ($variable_path: expr, $dust_crate: expr) => {{
        let module_names = $variable_path.module_names();
        let mut current = ($dust_crate, Span::default());

        for module_name in module_names {
            let module_path = Path::new_borrowed(module_name).unwrap();

            if let Some(next) = current.0.get_item(&module_path) {
                if let Item::Module(module) = &next.0 {
                    current = (module, next.1);
                }
            }
        }

        let item_name = Path::new_borrowed($variable_path.item_name()).unwrap();

        current.0.get_item(&item_name).cloned()
    }};
}

pub use find_item;
