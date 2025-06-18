use std::{marker::PhantomData, sync::Arc};

use serde::{
    Deserialize, Serialize,
    de::{VariantAccess, Visitor},
};

use crate::{Type, Value};

use super::Module;

#[derive(Clone, Debug, PartialEq, Serialize)]
pub enum Item<'a, C> {
    Constant { value: Value<C>, r#type: Type },
    Function(Arc<C>),
    Module(Module<'a, C>),
}

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
        let (variant, access) = data.variant()?;

        match variant {
            "Constant" => {
                let constant = access.struct_variant(
                    &["value", "r#type"],
                    ConstantVisitor {
                        _marker: PhantomData,
                    },
                )?;

                Ok(constant)
            }
            "Function" => {
                let function: Arc<C> = access.newtype_variant()?;

                Ok(Item::Function(function))
            }
            "Module" => {
                let module: Module<'de, C> = access.newtype_variant()?;

                Ok(Item::Module(module))
            }
            _ => Err(serde::de::Error::unknown_variant(
                variant,
                &["Constant", "Function", "Module"],
            )),
        }
    }
}

struct ConstantVisitor<'de, C> {
    _marker: PhantomData<&'de C>,
}

impl<'de, C: Deserialize<'de>> Visitor<'de> for ConstantVisitor<'de, C> {
    type Value = Item<'de, C>;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("a Constant variant")
    }

    fn visit_seq<S>(self, mut seq: S) -> Result<Self::Value, S::Error>
    where
        S: serde::de::SeqAccess<'de>,
    {
        let value: Value<C> = seq
            .next_element()?
            .ok_or_else(|| serde::de::Error::invalid_length(0, &self))?;
        let r#type: Type = seq
            .next_element()?
            .ok_or_else(|| serde::de::Error::invalid_length(1, &self))?;

        Ok(Item::Constant { value, r#type })
    }
}

impl<'de, C> Deserialize<'de> for Item<'de, C>
where
    C: 'de + Deserialize<'de>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let item_visitor = ItemVisitor {
            _marker: PhantomData,
        };

        deserializer.deserialize_enum("Item", &["Constant", "Function", "Module"], item_visitor)
    }
}
