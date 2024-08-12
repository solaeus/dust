//! Garbage-collecting context for variables.
use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};

use crate::{Identifier, Span, Type, Value};

/// Garbage-collecting context for variables.
#[derive(Debug, Clone)]
pub struct Context {
    variables: Arc<RwLock<HashMap<Identifier, (VariableData, Span)>>>,
    is_garbage_collected_to: Arc<RwLock<usize>>,
}

impl Context {
    pub fn new() -> Self {
        Self {
            variables: Arc::new(RwLock::new(HashMap::new())),
            is_garbage_collected_to: Arc::new(RwLock::new(0)),
        }
    }

    pub fn with_variables_from(other: &Self) -> Self {
        Self {
            variables: Arc::new(RwLock::new(other.variables.read().unwrap().clone())),
            is_garbage_collected_to: Arc::new(RwLock::new(
                *other.is_garbage_collected_to.read().unwrap(),
            )),
        }
    }

    pub fn variable_count(&self) -> usize {
        self.variables.read().unwrap().len()
    }

    pub fn contains(&self, identifier: &Identifier) -> bool {
        self.variables.read().unwrap().contains_key(identifier)
    }

    pub fn get(&self, identifier: &Identifier) -> Option<(VariableData, Span)> {
        self.variables.read().unwrap().get(identifier).cloned()
    }

    pub fn get_type(&self, identifier: &Identifier) -> Option<Type> {
        match self.variables.read().unwrap().get(identifier) {
            Some((VariableData::Type(r#type), _)) => Some(r#type.clone()),
            _ => None,
        }
    }

    pub fn get_variable_data(&self, identifier: &Identifier) -> Option<VariableData> {
        match self.variables.read().unwrap().get(identifier) {
            Some((variable_data, _)) => Some(variable_data.clone()),
            _ => None,
        }
    }

    pub fn get_value(&self, identifier: &Identifier) -> Option<Value> {
        match self.variables.read().unwrap().get(identifier) {
            Some((VariableData::Value(value), _)) => Some(value.clone()),
            _ => None,
        }
    }

    pub fn set_type(&self, identifier: Identifier, r#type: Type, position: Span) {
        log::trace!("Setting {identifier} to type {type} at {position:?}");

        self.variables
            .write()
            .unwrap()
            .insert(identifier, (VariableData::Type(r#type), position));
    }

    pub fn set_value(&self, identifier: Identifier, value: Value) {
        log::trace!("Setting {identifier} to value {value}");

        let mut variables = self.variables.write().unwrap();

        let last_position = variables
            .get(&identifier)
            .map(|(_, last_position)| *last_position)
            .unwrap_or_default();

        variables.insert(identifier, (VariableData::Value(value), last_position));
    }

    pub fn collect_garbage(&self, current_position: usize) {
        log::trace!("Collecting garbage up to {current_position}");

        let mut is_garbage_collected_to = self.is_garbage_collected_to.write().unwrap();

        if current_position < *is_garbage_collected_to {
            return;
        }

        let mut variables = self.variables.write().unwrap();

        variables.retain(|identifier, (_, last_used)| {
            let should_drop = current_position >= last_used.1;

            if should_drop {
                log::trace!("Removing {identifier}");
            }

            !should_drop
        });
        variables.shrink_to_fit();

        *is_garbage_collected_to = current_position;
    }

    pub fn update_last_position(&self, identifier: &Identifier, position: Span) -> bool {
        if let Some((_, last_position)) = self.variables.write().unwrap().get_mut(identifier) {
            *last_position = position;

            log::trace!("Updating {identifier}'s last position to {position:?}");

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

#[cfg(test)]
mod tests {
    use crate::vm::run_with_context;

    use super::*;

    #[test]
    fn context_removes_variables() {
        env_logger::builder().is_test(true).try_init().unwrap();

        let source = "
            x = 5
            y = 10
            z = x + y
            z
        ";
        let context = Context::new();

        run_with_context(source, context.clone()).unwrap();

        assert_eq!(context.variable_count(), 0);
    }

    #[test]
    fn garbage_collector_does_not_break_loops() {
        let source = "
            y = 1
            z = 0
            while z < 10 {
                z = z + y
            }
        ";
        let context = Context::new();

        run_with_context(source, context.clone()).unwrap();

        assert_eq!(context.variable_count(), 0);
    }
}
