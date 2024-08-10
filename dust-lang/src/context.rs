use std::collections::HashMap;

use crate::{Identifier, Type, Value};

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

    pub fn get_value(&self, identifier: &Identifier) -> Option<&Value> {
        match self.variables.get(identifier) {
            Some((VariableData::Value(value), _)) => Some(value),
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
}

impl Default for Context {
    fn default() -> Self {
        Self::new()
    }
}

pub enum VariableData {
    Value(Value),
    Type(Type),
}

#[derive(Default)]
pub struct UsageData {
    pub allowed_uses: u16,
    pub used: u16,
}
