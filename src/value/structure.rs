use std::{
    collections::BTreeMap,
    fmt::{self, Display, Formatter},
    marker::PhantomData,
    sync::Arc,
};

use serde::{
    de::{MapAccess, Visitor},
    ser::SerializeMap,
    Deserialize, Deserializer, Serialize, Serializer,
};

use crate::{Type, Value};

#[derive(Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
pub struct Structure(Arc<BTreeMap<String, (Option<Value>, Type)>>);

impl Structure {
    pub fn new(map: BTreeMap<String, (Option<Value>, Type)>) -> Self {
        Structure(Arc::new(map))
    }

    pub fn inner(&self) -> &BTreeMap<String, (Option<Value>, Type)> {
        &self.0
    }
}

impl Display for Structure {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        writeln!(f, "{{")?;

        for (key, (value_option, r#type)) in self.0.as_ref() {
            if let Some(value) = value_option {
                writeln!(f, "  {key} <{}> = {value}", r#type)?;
            } else {
                writeln!(f, "  {key} <{}>", r#type)?;
            }
        }
        write!(f, "}}")
    }
}

impl Serialize for Structure {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut map = serializer.serialize_map(Some(self.0.len()))?;

        for (key, (value, _type)) in self.0.iter() {
            map.serialize_entry(key, value)?;
        }

        map.end()
    }
}

struct StructureVisitor {
    marker: PhantomData<fn() -> Structure>,
}

impl StructureVisitor {
    fn new() -> Self {
        StructureVisitor {
            marker: PhantomData,
        }
    }
}

impl<'de> Visitor<'de> for StructureVisitor {
    type Value = Structure;

    fn expecting(&self, formatter: &mut Formatter) -> fmt::Result {
        formatter.write_str("key-value pairs")
    }

    fn visit_map<M>(self, mut access: M) -> std::result::Result<Structure, M::Error>
    where
        M: MapAccess<'de>,
    {
        let mut b_tree = BTreeMap::new();

        {
            while let Some((key, value)) = access.next_entry::<String, Value>()? {
                let r#type = value.r#type();

                b_tree.insert(key, (Some(value), r#type));
            }
        }

        Ok(Structure::new(b_tree))
    }
}

impl<'de> Deserialize<'de> for Structure {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_any(StructureVisitor::new())
    }
}
