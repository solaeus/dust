//! Garbage-collecting context for variables.
use std::collections::HashMap;

use crate::{Identifier, Type, Value};

/// Garbage-collecting context for variables.
#[derive(Debug, Clone)]
pub struct Context {
    pub variables: HashMap<Identifier, (VariableData, UsageData)>,
}

impl Context {
    pub fn new() -> Self {
        Self {
            variables: HashMap::new(),
        }
    }

    pub fn contains(&self, identifier: &Identifier) -> bool {
        self.variables.contains_key(identifier)
    }

    pub fn get(&self, identifier: &Identifier) -> Option<&(VariableData, UsageData)> {
        self.variables.get(identifier)
    }

    pub fn get_type(&self, identifier: &Identifier) -> Option<&Type> {
        match self.variables.get(identifier) {
            Some((VariableData::Type(r#type), _)) => Some(r#type),
            _ => None,
        }
    }

    pub fn use_value(&mut self, identifier: &Identifier) -> Option<&Value> {
        match self.variables.get_mut(identifier) {
            Some((VariableData::Value(value), usage_data)) => {
                usage_data.used += 1;

                Some(value)
            }
            _ => None,
        }
    }

    pub fn get_variable_data(&self, identifier: &Identifier) -> Option<&VariableData> {
        match self.variables.get(identifier) {
            Some((variable_data, _)) => Some(variable_data),
            _ => None,
        }
    }

    pub fn set_type(&mut self, identifier: Identifier, r#type: Type) {
        self.variables.insert(
            identifier,
            (VariableData::Type(r#type), UsageData::default()),
        );
    }

    pub fn set_value(&mut self, identifier: Identifier, value: Value) {
        self.variables.insert(
            identifier,
            (VariableData::Value(value), UsageData::default()),
        );
    }

    pub fn collect_garbage(&mut self) {
        self.variables
            .retain(|_, (_, usage_data)| usage_data.used < usage_data.allowed_uses);
    }

    pub fn add_allowed_use(&mut self, identifier: &Identifier) -> bool {
        if let Some((_, usage_data)) = self.variables.get_mut(identifier) {
            usage_data.allowed_uses += 1;

            true
        } else {
            false
        }
    }
}

impl Default for Context {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub enum VariableData {
    Value(Value),
    Type(Type),
}

#[derive(Default, Debug, Clone)]
pub struct UsageData {
    pub allowed_uses: u16,
    pub used: u16,
}
