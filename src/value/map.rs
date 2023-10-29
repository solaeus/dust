use serde::Serialize;
use std::{
    cmp::Ordering,
    collections::BTreeMap,
    fmt::{self, Display, Formatter},
    sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard},
};

use crate::{value::Value, List, Table};

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

    pub fn clone_from(other: &Self) -> Self {
        let mut new_map = BTreeMap::new();

        for (key, value) in other.inner().read().unwrap().iter() {
            new_map.insert(key.clone(), value.clone());
        }

        Map {
            variables: Arc::new(RwLock::new(new_map)),
        }
    }

    pub fn variables(&self) -> RwLockReadGuard<BTreeMap<String, Value>> {
        self.variables.read().unwrap()
    }

    pub fn variables_mut(&self) -> RwLockWriteGuard<BTreeMap<String, Value>> {
        self.variables.write().unwrap()
    }

    /// Removes an assigned variable.
    ///
    /// TODO: Support dot notation.
    pub fn remove(&mut self, key: &str) -> Option<Value> {
        self.variables.write().unwrap().remove(key)
    }

    /// Returns a reference to the inner BTreeMap.
    pub fn inner(&self) -> Arc<RwLock<BTreeMap<String, Value>>> {
        Arc::clone(&self.variables)
    }

    /// Returns the number of stored variables.
    pub fn len(&self) -> usize {
        self.variables.read().unwrap().len()
    }

    /// Returns true if the length is zero.
    pub fn is_empty(&self) -> bool {
        self.variables.read().unwrap().is_empty()
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
        write!(f, "{{\n")?;

        let variables = self.variables.read().unwrap().clone().into_iter();

        for (key, value) in variables {
            write!(f, "  {key} = {value}\n")?;
        }
        write!(f, "}}")
    }
}

impl From<&Table> for Map {
    fn from(value: &Table) -> Self {
        let map = Map::new();

        for (row_index, row) in value.rows().iter().enumerate() {
            map.variables_mut()
                .insert(
                    row_index.to_string(),
                    Value::List(List::with_items(row.clone())),
                )
                .unwrap();
        }

        map
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
