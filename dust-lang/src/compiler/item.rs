use std::{marker::PhantomData, sync::Arc};

use serde::{
    Deserialize, Serialize,
    de::{VariantAccess, Visitor},
};

use crate::Value;

use super::Module;

#[derive(Clone, Debug, PartialEq, Serialize)]
pub enum Item<'a, C> {
    Constant(Value<C>),
    Function(Arc<C>),
    Module(Module<'a, C>),
}

impl<'de, C> Deserialize<'de> for Item<'de, C>
where
    C: 'de + Deserialize<'de>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct ItemVisitor<'de, C> {
            _marker: PhantomData<&'de C>,
        }

        impl<'de, C: Deserialize<'de>> Visitor<'de> for ItemVisitor<'de, C> {
            type Value = Item<'de, C>;

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

        let item_visitor = ItemVisitor {
            _marker: PhantomData,
        };

        deserializer.deserialize_enum("Item", &["Constant", "Function", "Module"], item_visitor)
    }
}
