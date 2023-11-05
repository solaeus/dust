use serde::Serialize;
use std::{
    cmp::Ordering,
    collections::BTreeMap,
    fmt::{self, Display, Formatter},
    sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard},
};

use crate::{value::Value, List, Result, Table};

/// A collection dust variables comprised of key-value pairs.
///
/// The inner value is a BTreeMap in order to allow VariableMap instances to be sorted and compared
/// to one another.
#[derive(Clone, Debug)]
pub struct Map {
    variables: Arc<RwLock<BTreeMap<String, Value>>>,
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

        for (key, value) in other.variables()?.iter() {
            new_map.insert(key.clone(), value.clone());
        }

        Ok(Map {
            variables: Arc::new(RwLock::new(new_map)),
        })
    }

    pub fn variables(&self) -> Result<RwLockReadGuard<BTreeMap<String, Value>>> {
        Ok(self.variables.read()?)
    }

    pub fn variables_mut(&self) -> Result<RwLockWriteGuard<BTreeMap<String, Value>>> {
        Ok(self.variables.write()?)
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

        for (key, value) in variables {
            writeln!(f, "  {key} = {value}")?;
        }
        write!(f, "}}")
    }
}

impl From<&Table> for Result<Map> {
    fn from(value: &Table) -> Result<Map> {
        let map = Map::new();

        for (row_index, row) in value.rows().iter().enumerate() {
            map.variables_mut()?
                .insert(
                    row_index.to_string(),
                    Value::List(List::with_items(row.clone())),
                )
                .unwrap();
        }

        Ok(map)
    }
}

impl Serialize for Map {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.variables.serialize(serializer)
    }
}
