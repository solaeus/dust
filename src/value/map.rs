use serde::{ser::SerializeMap, Deserialize, Deserializer, Serialize};
use std::{
    cmp::Ordering,
    collections::BTreeMap,
    fmt::{self, Display, Formatter},
    sync::{Arc, RwLock, RwLockReadGuard},
};

use crate::{value::Value, Result, Type};

/// A collection dust variables comprised of key-value pairs.
///
/// The inner value is a BTreeMap in order to allow VariableMap instances to be sorted and compared
/// to one another.
#[derive(Clone, Debug)]
pub struct Map {
    variables: Arc<RwLock<BTreeMap<String, (Value, Type)>>>,
}

impl Map {
    /// Creates a new instace.
    pub fn new() -> Self {
        Map {
            variables: Arc::new(RwLock::new(BTreeMap::new())),
        }
    }

    pub fn clone_from(other: &Self) -> Result<Self> {
        let mut new_map = BTreeMap::new();

        for (key, (value, r#type)) in other.variables()?.iter() {
            new_map.insert(key.clone(), (value.clone(), r#type.clone()));
        }

        Ok(Map {
            variables: Arc::new(RwLock::new(new_map)),
        })
    }

    pub fn variables(&self) -> Result<RwLockReadGuard<BTreeMap<String, (Value, Type)>>> {
        Ok(self.variables.read()?)
    }

    pub fn set(
        &self,
        key: String,
        value: Value,
        r#type: Option<Type>,
    ) -> Result<Option<(Value, Type)>> {
        let value_type = r#type.unwrap_or(value.r#type());
        let previous = self
            .variables
            .write()?
            .insert(key, (value, value_type.clone()));

        Ok(previous)
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
        let left = self.variables.read().unwrap().clone().into_iter();
        let right = other.variables.read().unwrap().clone().into_iter();

        left.partial_cmp(right)
    }
}

impl Display for Map {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        writeln!(f, "{{")?;

        let variables = self.variables.read().unwrap().clone().into_iter();

        for (key, (value, _)) in variables {
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
        let variables = self.variables.read().unwrap();
        let mut map = serializer.serialize_map(Some(variables.len()))?;

        for (key, (value, _type)) in variables.iter() {
            map.serialize_entry(key, value)?;
        }

        map.end()
    }
}

impl<'de> Deserialize<'de> for Map {
    fn deserialize<D>(_deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        todo!()
    }
}
