use serde::{
    de::{MapAccess, Visitor},
    ser::SerializeMap,
    Deserialize, Serialize,
};
use std::{
    cmp::Ordering,
    collections::BTreeMap,
    fmt::{self, Display, Formatter},
    marker::PhantomData,
};

use crate::{value::Value, Result, Type};

/// A collection dust variables comprised of key-value pairs.
///
/// The inner value is a BTreeMap in order to allow VariableMap instances to be sorted and compared
/// to one another.
#[derive(Clone, Debug)]
pub struct Map {
    variables: BTreeMap<String, (Value, Type)>,
}

impl Map {
    /// Creates a new instace.
    pub fn new() -> Self {
        Map {
            variables: BTreeMap::new(),
        }
    }

    pub fn clone_from(other: &Self) -> Result<Self> {
        let mut variables = BTreeMap::new();

        for (key, (value, r#type)) in other.variables() {
            variables.insert(key.clone(), (value.clone(), r#type.clone()));
        }

        Ok(Map { variables })
    }

    pub fn variables(&self) -> &BTreeMap<String, (Value, Type)> {
        &self.variables
    }

    pub fn set(
        &mut self,
        key: String,
        value: Value,
        r#type: Option<Type>,
    ) -> Option<(Value, Type)> {
        let value_type = r#type.unwrap_or(value.r#type());

        self.variables.insert(key, (value, value_type.clone()))
    }

    pub fn unset_all(&mut self) -> Result<()> {
        for (_key, (value, r#_type)) in self.variables.iter_mut() {
            *value = Value::none();
        }

        Ok(())
    }

    pub fn clear(&mut self) -> Result<()> {
        self.variables.clear();

        Ok(())
    }
}

impl Default for Map {
    fn default() -> Self {
        Self::new()
    }
}

impl Eq for Map {}

impl PartialEq for Map {
    fn eq(&self, other: &Self) -> bool {
        self.variables.eq(&other.variables)
    }
}

impl Ord for Map {
    fn cmp(&self, other: &Self) -> Ordering {
        self.variables.cmp(&other.variables)
    }
}

impl PartialOrd for Map {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Display for Map {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        writeln!(f, "{{")?;

        for (key, (value, _)) in &self.variables {
            writeln!(f, "  {key} = {value}")?;
        }
        write!(f, "}}")
    }
}

impl Serialize for Map {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut map = serializer.serialize_map(Some(self.variables.len()))?;

        for (key, (value, _type)) in &self.variables {
            map.serialize_entry(&key, &value)?;
        }

        map.end()
    }
}

struct MapVisitor {
    marker: PhantomData<fn() -> Map>,
}

impl MapVisitor {
    fn new() -> Self {
        MapVisitor {
            marker: PhantomData,
        }
    }
}

impl<'de> Visitor<'de> for MapVisitor {
    type Value = Map;

    fn expecting(&self, formatter: &mut Formatter) -> fmt::Result {
        formatter.write_str("Valid map data.")
    }

    fn visit_map<M>(self, mut access: M) -> std::result::Result<Map, M::Error>
    where
        M: MapAccess<'de>,
    {
        let mut map = Map::new();

        {
            while let Some((key, value)) = access.next_entry::<String, Value>()? {
                map.set(key, value, None).unwrap();
            }
        }

        Ok(map)
    }
}

impl<'de> Deserialize<'de> for Map {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_any(MapVisitor::new())
    }
}
