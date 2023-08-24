use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeMap,
    fmt::{self, Display, Formatter},
};

use crate::{value::Value, Error, Result, Table, TOOL_LIST};

/// A context that stores its mappings in hash maps.
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

    /// Invokes built-in tools or user-defined functions based on the identifier and passes the     
    /// argument. Returns an error a tool is called with the wrong inputs or if the identifier does
    /// not match any tools or functions.
    pub fn call_function(&self, identifier: &str, argument: &Value) -> Result<Value> {
        for macro_item in TOOL_LIST {
            let valid_input_types = macro_item.info().inputs;

            if identifier == macro_item.info().identifier {
                let input_type = argument.value_type();

                if valid_input_types.contains(&input_type) {
                    return macro_item.run(argument);
                } else {
                    return Err(Error::MacroArgumentType {
                        macro_info: macro_item.info(),
                        actual: argument.clone(),
                    });
                }
            }
        }

        for (key, value) in &self.variables {
            if identifier == key {
                if let Ok(function) = value.as_function() {
                    let mut context = self.clone();

                    context.set_value("input", argument.clone())?;

                    return function.run_with_context(&mut context);
                }
            }
        }

        Err(Error::FunctionIdentifierNotFound(identifier.to_string()))
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
    pub fn set_value(&mut self, identifier: &str, value: Value) -> Result<()> {
        let split = identifier.split_once('.');

        if let Some((map_name, next_identifier)) = split {
            let get_value = self.variables.get_mut(map_name);

            if let Some(found_value) = get_value {
                if let Value::List(list) = found_value {
                    let index = if let Ok(index) = next_identifier.parse::<usize>() {
                        index
                    } else {
                        return Err(Error::expected_int(Value::String(
                            next_identifier.to_string(),
                        )));
                    };

                    let mut missing_elements = index - list.len() + 1;

                    while missing_elements > 0 {
                        list.push(value.clone());

                        missing_elements -= 1;
                    }

                    Ok(())
                } else if let Value::Map(map) = found_value {
                    map.set_value(next_identifier, value)
                } else {
                    Err(Error::ExpectedMap {
                        actual: found_value.clone(),
                    })
                }
            } else {
                let mut new_map = VariableMap::new();

                new_map.set_value(next_identifier, value)?;

                self.variables
                    .insert(map_name.to_string(), Value::Map(new_map));

                Ok(())
            }
        } else {
            self.variables.insert(identifier.to_string(), value);

            Ok(())
        }
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
            map.set_value(&row_index.to_string(), Value::List(row.clone()))
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

        map.set_value("x", Value::Integer(1)).unwrap();

        assert_eq!(Value::Integer(1), map.get_value("x").unwrap().unwrap());
    }

    #[test]
    fn get_and_set_nested_maps() {
        let mut map = VariableMap::new();

        map.set_value("x", Value::Map(VariableMap::new())).unwrap();
        map.set_value("x.x", Value::Map(VariableMap::new()))
            .unwrap();
        map.set_value("x.x.x", Value::Map(VariableMap::new()))
            .unwrap();
        map.set_value("x.x.x.x", Value::Map(VariableMap::new()))
            .unwrap();

        assert_eq!(
            Value::Map(VariableMap::new()),
            map.get_value("x.x.x.x").unwrap().unwrap()
        );
    }
}
