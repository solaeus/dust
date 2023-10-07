use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeMap,
    fmt::{self, Display, Formatter},
};

use crate::{value::Value, Error, Result, Table};

/// A collection dust variables comprised of key-value pairs.
///
/// The inner value is a BTreeMap in order to allow VariableMap instances to be sorted and compared
/// to one another.
#[derive(Clone, Debug, PartialEq, PartialOrd, Ord, Eq, Serialize, Deserialize)]
pub struct VariableMap {
    variables: BTreeMap<String, Value>,
}

impl VariableMap {
    /// Creates a new instace.
    pub fn new() -> Self {
        VariableMap {
            variables: BTreeMap::new(),
        }
    }

    /// Returns a Value assigned to the identifer, allowing dot notation to retrieve Values that are     /// nested in Lists or Maps. Returns None if there is no variable with a key matching the            /// identifier. Returns an error if a Map or List is indexed incorrectly.
    pub fn get_value(&self, identifier: &str) -> Result<Option<Value>> {
        let split = identifier.rsplit_once('.');
        let (found_value, next_identifier) = if let Some((identifier, next_identifier)) = split {
            if identifier.contains('.') {
                (self.get_value(identifier)?, next_identifier)
            } else {
                (self.variables.get(identifier).cloned(), next_identifier)
            }
        } else {
            return Ok(self.variables.get(identifier).cloned());
        };

        if let Some(value) = found_value {
            if let Value::List(list) = value {
                let index = if let Ok(index) = next_identifier.parse::<usize>() {
                    index
                } else {
                    return Err(Error::expected_int(Value::String(
                        next_identifier.to_string(),
                    )));
                };

                Ok(list.get(index).cloned())
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
            let get_value = self.variables.get_mut(identifier);

            if let Some(found_value) = get_value {
                if let Value::List(list) = found_value {
                    let index = if let Ok(index) = next_identifier.parse::<usize>() {
                        index
                    } else {
                        return Err(Error::expected_int(Value::String(
                            next_identifier.to_string(),
                        )));
                    };

                    let mut missing_elements = index.saturating_sub(list.len()) + 1;

                    while missing_elements > 0 {
                        list.push(value.clone());

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
                let mut new_map = VariableMap::new();

                new_map.set_value(next_identifier.to_string(), value)?;

                self.variables
                    .insert(identifier.to_string(), Value::Map(new_map));

                Ok(())
            }
        } else {
            self.variables.insert(key.to_string(), value);

            Ok(())
        }
    }

    /// Removes and assignmed variable.
    ///
    /// TODO: Support dot notation.
    pub fn remove(&mut self, key: &str) -> Option<Value> {
        self.variables.remove(key)
    }

    /// Returns a reference to the inner BTreeMap.
    pub fn inner(&self) -> &BTreeMap<String, Value> {
        &self.variables
    }

    /// Returns the number of stored variables.
    pub fn len(&self) -> usize {
        self.variables.len()
    }

    /// Returns true if the length is zero.
    pub fn is_empty(&self) -> bool {
        self.variables.is_empty()
    }
}

impl Default for VariableMap {
    fn default() -> Self {
        Self::new()
    }
}

impl Display for VariableMap {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        Table::from(self).fmt(f)
    }
}

impl From<&Table> for VariableMap {
    fn from(value: &Table) -> Self {
        let mut map = VariableMap::new();

        for (row_index, row) in value.rows().iter().enumerate() {
            map.set_value(row_index.to_string(), Value::List(row.clone()))
                .unwrap();
        }

        map
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get_and_set_simple_value() {
        let mut map = VariableMap::new();

        map.set_value("x".to_string(), Value::Integer(1)).unwrap();

        assert_eq!(Value::Integer(1), map.get_value("x").unwrap().unwrap());
    }

    #[test]
    fn get_and_set_nested_maps() {
        let mut map = VariableMap::new();

        map.set_value("x".to_string(), Value::Map(VariableMap::new()))
            .unwrap();
        map.set_value("x.x".to_string(), Value::Map(VariableMap::new()))
            .unwrap();
        map.set_value("x.x.x".to_string(), Value::Map(VariableMap::new()))
            .unwrap();
        map.set_value("x.x.x.x".to_string(), Value::Map(VariableMap::new()))
            .unwrap();

        assert_eq!(
            Value::Map(VariableMap::new()),
            map.get_value("x.x.x.x").unwrap().unwrap()
        );
    }
}
