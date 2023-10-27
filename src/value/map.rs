use serde::Serialize;
use std::{
    cmp::Ordering,
    collections::BTreeMap,
    fmt::{self, Display, Formatter},
    sync::{Arc, RwLock},
};

use crate::{value::Value, Error, List, Result, Table};

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

    /// Returns a Value assigned to the identifer, allowing dot notation to retrieve Values that are     /// nested in Lists or Maps. Returns None if there is no variable with a key matching the            /// identifier. Returns an error if a Map or List is indexed incorrectly.
    pub fn get_value(&self, identifier: &str) -> Result<Option<Value>> {
        let variables = self.variables.read().unwrap();

        let split = identifier.rsplit_once('.');
        let (found_value, next_identifier) = if let Some((identifier, next_identifier)) = split {
            if identifier.contains('.') {
                (self.get_value(identifier)?, next_identifier)
            } else {
                (variables.get(identifier).cloned(), next_identifier)
            }
        } else {
            return Ok(variables.get(identifier).cloned());
        };

        if let Some(value) = found_value {
            if let Value::List(list) = value {
                let index = if let Ok(index) = next_identifier.parse::<usize>() {
                    index
                } else {
                    return Err(Error::ExpectedInt {
                        actual: Value::String(next_identifier.to_string()),
                    });
                };

                Ok(list.items().get(index).cloned())
            } else if let Value::Map(map) = value {
                map.get_value(next_identifier)
            } else {
                Ok(Some(value))
            }
        } else {
            Ok(None)
        }
    }

    /// Assigns a variable with a Value and the identifier as its key, allowing dot notation to
    /// assign nested lists and maps. Returns an error if a List or Map is indexed incorrectly.
    pub fn set_value(&mut self, key: String, value: Value) -> Result<()> {
        let split = key.split_once('.');

        if let Some((identifier, next_identifier)) = split {
            let mut variables = self.variables.write().unwrap();
            let get_value = variables.get_mut(identifier);

            if let Some(found_value) = get_value {
                if let Value::List(list) = found_value {
                    let index = if let Ok(index) = next_identifier.parse::<usize>() {
                        index
                    } else {
                        return Err(Error::ExpectedInt {
                            actual: Value::String(next_identifier.to_string()),
                        });
                    };

                    let mut missing_elements = index.saturating_sub(list.items().len()) + 1;
                    let mut items = list.items_mut();

                    while missing_elements > 0 {
                        items.push(value.clone());

                        missing_elements -= 1;
                    }

                    Ok(())
                } else if let Value::Map(map) = found_value {
                    map.set_value(next_identifier.to_string(), value)
                } else {
                    Err(Error::ExpectedMap {
                        actual: found_value.clone(),
                    })
                }
            } else {
                let mut new_map = Map::new();

                new_map.set_value(next_identifier.to_string(), value)?;

                self.variables
                    .write()
                    .unwrap()
                    .insert(identifier.to_string(), Value::Map(new_map));

                Ok(())
            }
        } else {
            self.variables
                .write()
                .unwrap()
                .insert(key.to_string(), value);

            Ok(())
        }
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
        let mut map = Map::new();

        for (row_index, row) in value.rows().iter().enumerate() {
            map.set_value(
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get_and_set_simple_value() {
        let mut map = Map::new();

        map.set_value("x".to_string(), Value::Integer(1)).unwrap();

        assert_eq!(Value::Integer(1), map.get_value("x").unwrap().unwrap());
    }

    #[test]
    fn get_and_set_nested_maps() {
        let mut map = Map::new();

        map.set_value("x".to_string(), Value::Map(Map::new()))
            .unwrap();
        map.set_value("x.x".to_string(), Value::Map(Map::new()))
            .unwrap();
        map.set_value("x.x.x".to_string(), Value::Map(Map::new()))
            .unwrap();
        map.set_value("x.x.x.x".to_string(), Value::Map(Map::new()))
            .unwrap();

        assert_eq!(
            Value::Map(Map::new()),
            map.get_value("x.x.x.x").unwrap().unwrap()
        );
    }
}
