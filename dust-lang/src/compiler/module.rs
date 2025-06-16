use std::{collections::HashMap, marker::PhantomData};

use serde::{Deserialize, Serialize};

use crate::Span;

use super::{Item, Path};

#[derive(Clone, Debug, Default, PartialEq, Serialize)]
pub struct Module<'a, C> {
    pub items: HashMap<Path<'a>, (Item<'a, C>, Span)>,
}

impl<'a, C> Module<'a, C> {
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

    pub fn find_item<'b>(&'b self, variable_path: &'b Path<'a>) -> Option<&'b (Item<'a, C>, Span)> {
        let mut current_module = self;

        for module_name in variable_path.module_names() {
            println!("{module_name}");

            if let Some((Item::Module(module), _)) = current_module.items.get(&module_name) {
                current_module = module;
            } else {
                return None;
            }
        }

        let item_name = variable_path.item_name();

        current_module.items.get(&item_name)
    }
}

impl<'de, C> Deserialize<'de> for Module<'de, C>
where
    C: 'de + Deserialize<'de>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct ModuleVisitor<'a, C> {
            marker: PhantomData<&'a C>,
        }

        impl<'de, C: Deserialize<'de>> serde::de::Visitor<'de> for ModuleVisitor<'de, C> {
            type Value = Module<'de, C>;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("a Module struct")
            }

            fn visit_map<M>(self, mut map: M) -> Result<Self::Value, M::Error>
            where
                M: serde::de::MapAccess<'de>,
            {
                let mut items = HashMap::new();

                while let Some((key, value)) =
                    map.next_entry::<Path<'de>, (Item<'de, C>, Span)>()?
                {
                    items.insert(key, value);
                }

                Ok(Module { items })
            }
        }

        deserializer.deserialize_struct(
            "Module",
            &["items"],
            ModuleVisitor {
                marker: PhantomData,
            },
        )
    }
}
