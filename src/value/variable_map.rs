use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeMap,
    fmt::{self, Display, Formatter},
};

use crate::{value::Value, Error, Result, Table, MACRO_LIST};

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

    pub fn call_function(&self, identifier: &str, argument: &Value) -> Result<Value> {
        for macro_item in MACRO_LIST {
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
                    let mut context = VariableMap::new();

                    context.set_value("input", argument.clone())?;

                    return function.run_with_context(&mut context);
                }
            }
        }

        Err(Error::FunctionIdentifierNotFound(identifier.to_string()))
    }

    pub fn get_value(&self, identifier: &str) -> Result<Option<Value>> {
        let split = identifier.split_once('.');

        if let Some((identifier, next_identifier)) = split {
            if let Some(value) = self.variables.get(identifier) {
                if let Value::Map(map) = value {
                    map.get_value(next_identifier)
                } else if let Value::List(list) = value {
                    let index = if let Ok(index) = next_identifier.parse::<usize>() {
                        index
                    } else {
                        return Err(Error::ExpectedInt {
                            actual: Value::String(next_identifier.to_string()),
                        });
                    };
                    let value = list.get(index);

                    Ok(value.cloned())
                } else {
                    Err(Error::ExpectedMap {
                        actual: value.clone(),
                    })
                }
            } else {
                Ok(None)
            }
        } else {
            let value = self.variables.get(identifier);

            if let Some(value) = value {
                Ok(Some(value.clone()))
            } else {
                Ok(None)
            }
        }
    }

    pub fn set_value(&mut self, identifier: &str, value: Value) -> Result<()> {
        let split = identifier.split_once('.');

        if let Some((map_name, next_identifier)) = split {
            let get_value = self.variables.get_mut(map_name);

            if let Some(map_value) = get_value {
                if let Value::Map(map) = map_value {
                    map.set_value(next_identifier, value)
                } else {
                    Err(Error::ExpectedMap {
                        actual: map_value.clone(),
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
