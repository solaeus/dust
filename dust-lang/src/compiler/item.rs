use std::{marker::PhantomData, sync::Arc};

use serde::{Deserialize, Serialize, de::VariantAccess};

use crate::{Chunk, ConcreteValue};

use super::Module;

#[derive(Clone, Debug, PartialEq, Serialize)]
pub enum Item<'a> {
    Constant(ConcreteValue),
    Function(Arc<Chunk>),
    Module(Module<'a>),
}

impl<'a, 'de: 'a> Deserialize<'de> for Item<'a> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let item_visitor = ItemVisitor {
            _marker: std::marker::PhantomData,
        };

        deserializer.deserialize_enum("Item", &["Constant", "Function", "Module"], item_visitor)
    }
}

struct ItemVisitor<'a> {
    _marker: PhantomData<&'a ()>,
}

impl<'a, 'de: 'a> serde::de::Visitor<'de> for ItemVisitor<'a> {
    type Value = Item<'a>;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("an Item enum")
    }

    fn visit_enum<A>(self, data: A) -> Result<Self::Value, A::Error>
    where
        A: serde::de::EnumAccess<'de>,
    {
        let (variant, value) = data.variant()?;

        match variant {
            "Constant" => Ok(Item::Constant(value.newtype_variant()?)),
            "Function" => Ok(Item::Function(Arc::new(value.newtype_variant()?))),
            "Module" => Ok(Item::Module(value.newtype_variant()?)),
            _ => Err(serde::de::Error::unknown_variant(
                variant,
                &["Constant", "Function", "Module"],
            )),
        }
    }
}
