//! Garbage-collecting context for variables.
use std::{
    collections::HashMap,
    sync::{Arc, PoisonError as StdPoisonError, RwLock, RwLockWriteGuard},
};

use crate::{ast::Span, Constructor, Identifier, StructType, Type, Value};

pub type Variables = HashMap<Identifier, (ContextData, Span)>;

/// Garbage-collecting context for variables.
#[derive(Debug, Clone)]
pub struct Context {
    variables: Arc<RwLock<Variables>>,
}

impl Context {
    pub fn new() -> Self {
        Self {
            variables: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Creates a deep copy of another context.
    pub fn with_data_from(other: &Self) -> Self {
        Self {
            variables: Arc::new(RwLock::new(other.variables.read().unwrap().clone())),
        }
    }

    /// Returns the number of variables in the context.
    pub fn variable_count(&self) -> usize {
        self.variables.read().unwrap().len()
    }

    /// Returns a boolean indicating whether the context contains the variable.
    pub fn contains(&self, identifier: &Identifier) -> bool {
        self.variables.read().unwrap().contains_key(identifier)
    }

    /// Returns the full VariableData and Span if the context contains the given identifier.
    pub fn get(&self, identifier: &Identifier) -> Option<(ContextData, Span)> {
        self.variables.read().unwrap().get(identifier).cloned()
    }

    /// Returns the type of the variable with the given identifier.
    pub fn get_type(&self, identifier: &Identifier) -> Option<Type> {
        match self.variables.read().unwrap().get(identifier) {
            Some((ContextData::VariableType(r#type), _)) => Some(r#type.clone()),
            Some((ContextData::VariableValue(value), _)) => Some(value.r#type()),
            Some((ContextData::ConstructorType(struct_type), _)) => {
                Some(Type::Struct(struct_type.clone()))
            }
            _ => None,
        }
    }

    /// Returns the VariableData of the variable with the given identifier.
    pub fn get_data(&self, identifier: &Identifier) -> Option<ContextData> {
        match self.variables.read().unwrap().get(identifier) {
            Some((variable_data, _)) => Some(variable_data.clone()),
            _ => None,
        }
    }

    /// Returns the value of the variable with the given identifier.
    pub fn get_variable_value(&self, identifier: &Identifier) -> Option<Value> {
        match self.variables.read().unwrap().get(identifier) {
            Some((ContextData::VariableValue(value), _)) => Some(value.clone()),
            _ => None,
        }
    }

    /// Returns the constructor associated with the given identifier.
    pub fn get_constructor(&self, identifier: &Identifier) -> Option<Constructor> {
        match self.variables.read().unwrap().get(identifier) {
            Some((ContextData::Constructor(constructor), _)) => Some(constructor.clone()),
            _ => None,
        }
    }

    /// Sets a variable to a type, with a position given for garbage collection.
    pub fn set_variable_type(&self, identifier: Identifier, r#type: Type, position: Span) {
        log::trace!("Setting {identifier} to type {type} at {position:?}");

        self.variables
            .write()
            .unwrap()
            .insert(identifier, (ContextData::VariableType(r#type), position));
    }

    /// Sets a variable to a value.
    pub fn set_variable_value(&self, identifier: Identifier, value: Value) {
        log::trace!("Setting {identifier} to value {value}");

        let mut variables = self.variables.write().unwrap();

        let last_position = variables
            .get(&identifier)
            .map(|(_, last_position)| *last_position)
            .unwrap_or_default();

        variables.insert(
            identifier,
            (ContextData::VariableValue(value), last_position),
        );
    }

    /// Associates a constructor with an identifier.
    pub fn set_constructor(&self, identifier: Identifier, constructor: Constructor) {
        log::trace!("Setting {identifier} to constructor {constructor}");

        let mut variables = self.variables.write().unwrap();

        let last_position = variables
            .get(&identifier)
            .map(|(_, last_position)| *last_position)
            .unwrap_or_default();

        variables.insert(
            identifier,
            (ContextData::Constructor(constructor), last_position),
        );
    }

    /// Associates a constructor type with an identifier.
    pub fn set_constructor_type(
        &self,
        identifier: Identifier,
        struct_type: StructType,
        position: Span,
    ) {
        log::trace!("Setting {identifier} to constructor of type {struct_type}");

        let mut variables = self.variables.write().unwrap();

        variables.insert(
            identifier,
            (ContextData::ConstructorType(struct_type), position),
        );
    }

    /// Collects garbage up to the given position, removing all variables with lesser positions.
    pub fn collect_garbage(&self, position: Span) {
        log::trace!("Collecting garbage up to {position:?}");

        let mut variables = self.variables.write().unwrap();

        variables.retain(|identifier, (_, last_used)| {
            let should_drop = position.0 > last_used.0 && position.1 > last_used.1;

            if should_drop {
                log::trace!("Removing {identifier}");
            }

            !should_drop
        });
        variables.shrink_to_fit();
    }

    /// Updates a variable's last known position, allowing it to live longer in the program.
    /// Returns a boolean indicating whether the variable was found.
    pub fn update_last_position(&self, identifier: &Identifier, position: Span) -> bool {
        if let Some((_, last_position)) = self.variables.write().unwrap().get_mut(identifier) {
            *last_position = position;

            log::trace!("Updating {identifier}'s last position to {position:?}");

            true
        } else {
            false
        }
    }

    /// Recovers the context from a poisoned state by recovering data from an error.
    ///
    /// This method is not used. The context's other methods do not return poison errors because
    /// they are infallible.
    pub fn _recover_from_poison(&mut self, error: &ContextPoisonError) {
        log::debug!("Context is recovering from poison error");

        let recovered = error.get_ref();
        let mut new_variables = HashMap::new();

        for (identifier, (variable_data, position)) in recovered.iter() {
            new_variables.insert(identifier.clone(), (variable_data.clone(), *position));
        }

        self.variables = Arc::new(RwLock::new(new_variables));
    }
}

impl Default for Context {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub enum ContextData {
    Constructor(Constructor),
    ConstructorType(StructType),
    VariableValue(Value),
    VariableType(Type),
}

pub type ContextPoisonError<'err> = StdPoisonError<RwLockWriteGuard<'err, Variables>>;

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
