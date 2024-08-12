//! Garbage-collecting context for variables.
use std::collections::HashMap;

use crate::{Identifier, Type, Value};

/// Garbage-collecting context for variables.
#[derive(Debug, Clone)]
pub struct Context {
    variables: HashMap<Identifier, (VariableData, UsageData)>,
    is_garbage_collected: bool,
}

impl Context {
    pub fn new() -> Self {
        Self {
            variables: HashMap::new(),
            is_garbage_collected: true,
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

    pub fn get_variable_data(&self, identifier: &Identifier) -> Option<&VariableData> {
        match self.variables.get(identifier) {
            Some((variable_data, _)) => Some(variable_data),
            _ => None,
        }
    }

    pub fn get_value(&self, identifier: &Identifier) -> Option<&Value> {
        match self.variables.get(identifier) {
            Some((VariableData::Value(value), _)) => Some(value),
            _ => None,
        }
    }

    pub fn use_value(&mut self, identifier: &Identifier) -> Option<&Value> {
        self.is_garbage_collected = false;

        match self.variables.get_mut(identifier) {
            Some((VariableData::Value(value), usage_data)) => {
                usage_data.used += 1;

                Some(value)
            }
            _ => None,
        }
    }

    pub fn set_type(&mut self, identifier: Identifier, r#type: Type) {
        self.is_garbage_collected = false;

        self.variables.insert(
            identifier,
            (VariableData::Type(r#type), UsageData::default()),
        );
    }

    pub fn set_value(&mut self, identifier: Identifier, value: Value) {
        self.is_garbage_collected = false;

        self.variables.insert(
            identifier,
            (VariableData::Value(value), UsageData::default()),
        );
    }

    pub fn collect_garbage(&mut self) {
        if !self.is_garbage_collected {
            self.variables
                .retain(|_, (_, usage_data)| usage_data.used < usage_data.allowed_uses);
            self.variables.shrink_to_fit();
        }
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

#[derive(Debug, Clone)]
pub struct UsageData {
    pub allowed_uses: u16,
    pub used: u16,
}

impl Default for UsageData {
    fn default() -> Self {
        Self {
            allowed_uses: 1,
            used: 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::vm::run_with_context;

    use super::*;

    #[test]
    fn context_removes_unused_variables() {
        let source = "
            x = 5
            y = 10
            z = x + y
        ";
        let mut context = Context::new();

        run_with_context(source, &mut context).unwrap();

        assert_eq!(context.variables.len(), 1);
    }
}
