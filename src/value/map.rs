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
    sync::{Arc, RwLock, RwLockReadGuard},
};

use crate::{value::Value, Result, Structure, Type};

/// A collection dust variables comprised of key-value pairs.
///
/// The inner value is a BTreeMap in order to allow VariableMap instances to be sorted and compared
/// to one another.
#[derive(Clone, Debug)]
pub struct Map {
    variables: Arc<RwLock<BTreeMap<String, (Value, Type)>>>,
    structure: Option<Structure>,
}

impl Map {
    /// Creates a new instace.
    pub fn new() -> Self {
        Map {
            variables: Arc::new(RwLock::new(BTreeMap::new())),
            structure: None,
        }
    }

    pub fn with_variables(variables: BTreeMap<String, (Value, Type)>) -> Self {
        Map {
            variables: Arc::new(RwLock::new(variables)),
            structure: None,
        }
    }

    pub fn clone_from(other: &Self) -> Result<Self> {
        let mut new_map = BTreeMap::new();

        for (key, (value, r#type)) in other.variables()?.iter() {
            new_map.insert(key.clone(), (value.clone(), r#type.clone()));
        }

        Ok(Map {
            variables: Arc::new(RwLock::new(new_map)),
            structure: other.structure.clone(),
        })
    }

    pub fn variables(&self) -> Result<RwLockReadGuard<BTreeMap<String, (Value, Type)>>> {
        Ok(self.variables.read()?)
    }

    pub fn set(&self, key: String, value: Value) -> Result<Option<(Value, Type)>> {
        log::info!("Setting variable {key} = {value}");

        let value_type = value.r#type();
        let previous = self
            .variables
            .write()?
            .insert(key, (value, value_type.clone()));

        Ok(previous)
    }

    pub fn set_type(&self, key: String, r#type: Type) -> Result<Option<(Value, Type)>> {
        log::info!("Setting type {key} = {}", r#type);

        let previous = self.variables.write()?.insert(key, (Value::none(), r#type));

        Ok(previous)
    }

    pub fn unset_all(&self) -> Result<()> {
        for (_key, (value, r#_type)) in self.variables.write()?.iter_mut() {
            *value = Value::none();
        }

        Ok(())
    }

    pub fn clear(&self) -> Result<()> {
        self.variables.write()?.clear();

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
        let left = self.variables.read().unwrap().clone().into_iter();
        let right = other.variables.read().unwrap().clone().into_iter();

        left.eq(right)
    }
}

impl Ord for Map {
    fn cmp(&self, other: &Self) -> Ordering {
        let left = self.variables.read().unwrap().clone().into_iter();
        let right = other.variables.read().unwrap().clone().into_iter();

        left.cmp(right)
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

        let variables = self.variables.read().unwrap().clone().into_iter();

        for (key, (value, value_type)) in variables {
            writeln!(f, "    {key} <{value_type}> = {value}")?;
        }
        write!(f, "}}")
    }
}

impl Serialize for Map {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let variables = self.variables.read().unwrap();
        let mut map = serializer.serialize_map(Some(variables.len()))?;

        for (key, (value, _type)) in variables.iter() {
            map.serialize_entry(key, value)?;
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
        formatter.write_str("Any valid whale data.")
    }

    fn visit_map<M>(self, mut access: M) -> std::result::Result<Map, M::Error>
    where
        M: MapAccess<'de>,
    {
        let map = Map::new();

        {
            while let Some((key, value)) = access.next_entry::<String, Value>()? {
                map.set(key, value).unwrap();
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
