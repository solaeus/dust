//! Garbage-collecting context for variables.
use std::{
    collections::HashMap,
    fmt::{self, Display, Formatter},
    sync::{Arc, PoisonError, RwLock, RwLockReadGuard, RwLockWriteGuard},
};

use crate::{Constructor, Identifier, StructType, Type, Value};

pub type Associations = HashMap<Identifier, (ContextData, usize)>;

/// Garbage-collecting context for variables.
#[derive(Debug, Clone)]
pub struct Context {
    associations: Arc<RwLock<Associations>>,
    parent: Option<Box<Context>>,
}

impl Context {
    pub fn new() -> Self {
        Self::with_data(HashMap::new())
    }

    pub fn with_data(data: Associations) -> Self {
        Self {
            associations: Arc::new(RwLock::new(data)),
            parent: None,
        }
    }

    /// Creates a deep copy of another context.
    pub fn with_data_from(other: &Self) -> Result<Self, ContextError> {
        Ok(Self::with_data(other.associations.read()?.clone()))
    }

    pub fn create_child(&self) -> Self {
        Self {
            associations: Arc::new(RwLock::new(HashMap::new())),
            parent: Some(Box::new(self.clone())),
        }
    }

    /// Returns the number of associated identifiers in the context.
    pub fn association_count(&self) -> Result<usize, ContextError> {
        Ok(self.associations.read()?.len())
    }

    /// Returns a boolean indicating whether the identifier is in the context.
    pub fn contains(&self, identifier: &Identifier) -> Result<bool, ContextError> {
        if self.associations.read()?.contains_key(identifier) {
            Ok(true)
        } else if let Some(parent) = &self.parent {
            parent.contains(identifier)
        } else {
            Ok(false)
        }
    }

    /// Returns the full ContextData and Span if the context contains the given identifier.
    pub fn get(
        &self,
        identifier: &Identifier,
    ) -> Result<Option<(ContextData, usize)>, ContextError> {
        if let Some((variable_data, position)) = self.associations.read()?.get(identifier) {
            Ok(Some((variable_data.clone(), *position)))
        } else if let Some(parent) = &self.parent {
            parent.get(identifier)
        } else {
            Ok(None)
        }
    }

    /// Returns the type associated with the given identifier.
    pub fn get_type(&self, identifier: &Identifier) -> Result<Option<Type>, ContextError> {
        match self.associations.read()?.get(identifier) {
            Some((ContextData::VariableType(r#type), _)) => return Ok(Some(r#type.clone())),
            Some((ContextData::VariableValue(value), _)) => return Ok(Some(value.r#type())),
            Some((ContextData::ConstructorType(struct_type), _)) => {
                return Ok(Some(Type::Struct(struct_type.clone())))
            }
            _ => {}
        }

        if let Some(parent) = &self.parent {
            parent.get_type(identifier)
        } else {
            Ok(None)
        }
    }

    /// Returns the ContextData associated with the identifier.
    pub fn get_data(&self, identifier: &Identifier) -> Result<Option<ContextData>, ContextError> {
        if let Some((variable_data, _)) = self.associations.read()?.get(identifier) {
            Ok(Some(variable_data.clone()))
        } else if let Some(parent) = &self.parent {
            parent.get_data(identifier)
        } else {
            Ok(None)
        }
    }

    /// Returns the value associated with the identifier.
    pub fn get_variable_value(
        &self,
        identifier: &Identifier,
    ) -> Result<Option<Value>, ContextError> {
        if let Some((ContextData::VariableValue(value), _)) =
            self.associations.read()?.get(identifier)
        {
            Ok(Some(value.clone()))
        } else if let Some(parent) = &self.parent {
            parent.get_variable_value(identifier)
        } else {
            Ok(None)
        }
    }

    /// Returns the constructor associated with the identifier.
    pub fn get_constructor(
        &self,
        identifier: &Identifier,
    ) -> Result<Option<Constructor>, ContextError> {
        if let Some((ContextData::Constructor(constructor), _)) =
            self.associations.read()?.get(identifier)
        {
            Ok(Some(constructor.clone()))
        } else if let Some(parent) = &self.parent {
            parent.get_constructor(identifier)
        } else {
            Ok(None)
        }
    }

    /// Returns the constructor type associated with the identifier.
    pub fn get_constructor_type(
        &self,
        identifier: &Identifier,
    ) -> Result<Option<StructType>, ContextError> {
        let read_associations = self.associations.read()?;

        if let Some((context_data, _)) = read_associations.get(identifier) {
            match context_data {
                ContextData::Constructor(constructor) => Ok(Some(constructor.struct_type.clone())),
                ContextData::ConstructorType(struct_type) => Ok(Some(struct_type.clone())),
                _ => Ok(None),
            }
        } else if let Some(parent) = &self.parent {
            parent.get_constructor_type(identifier)
        } else {
            Ok(None)
        }
    }

    /// Associates an identifier with a variable type, with a position given for garbage collection.
    pub fn set_variable_type(
        &self,
        identifier: Identifier,
        r#type: Type,
    ) -> Result<(), ContextError> {
        log::trace!("Setting {identifier} to type {type}.");

        let mut associations = self.associations.write()?;
        let last_position = associations
            .get(&identifier)
            .map(|(_, last_position)| *last_position)
            .unwrap_or_default();

        associations.insert(
            identifier,
            (ContextData::VariableType(r#type), last_position),
        );

        Ok(())
    }

    /// Associates an identifier with a variable value.
    pub fn set_variable_value(
        &self,
        identifier: Identifier,
        value: Value,
    ) -> Result<(), ContextError> {
        log::trace!("Setting {identifier} to value {value}");

        let mut associations = self.associations.write()?;
        let last_position = associations
            .get(&identifier)
            .map(|(_, last_position)| *last_position)
            .unwrap_or_default();

        associations.insert(
            identifier,
            (ContextData::VariableValue(value), last_position),
        );

        Ok(())
    }

    /// Associates an identifier with a constructor.
    pub fn set_constructor(
        &self,
        identifier: Identifier,
        constructor: Constructor,
    ) -> Result<(), ContextError> {
        log::trace!("Setting {identifier} to constructor {constructor:?}");

        let mut associations = self.associations.write()?;
        let last_position = associations
            .get(&identifier)
            .map(|(_, last_position)| *last_position)
            .unwrap_or_default();

        associations.insert(
            identifier,
            (ContextData::Constructor(constructor), last_position),
        );

        Ok(())
    }

    /// Associates an identifier with a constructor type, with a position given for garbage
    /// collection.
    pub fn set_constructor_type(
        &self,
        identifier: Identifier,
        struct_type: StructType,
    ) -> Result<(), ContextError> {
        log::trace!("Setting {identifier} to constructor of type {struct_type}");

        let mut variables = self.associations.write()?;
        let last_position = variables
            .get(&identifier)
            .map(|(_, last_position)| *last_position)
            .unwrap_or_default();

        variables.insert(
            identifier,
            (ContextData::ConstructorType(struct_type), last_position),
        );

        Ok(())
    }

    /// Collects garbage up to the given position, removing all variables with lesser positions.
    pub fn collect_garbage(&self, position: usize) -> Result<(), ContextError> {
        log::trace!("Collecting garbage up to {position}");

        let mut variables = self.associations.write()?;

        variables.retain(|identifier, (_, last_used)| {
            let should_drop = position >= *last_used;

            if should_drop {
                log::trace!("Removing {identifier}");
            }

            !should_drop
        });
        variables.shrink_to_fit();

        Ok(())
    }

    /// Updates an associated identifier's last known position, allowing it to live longer in the
    /// program. Returns a boolean indicating whether the identifier was found. If the identifier is
    /// not found in the current context, the parent context is searched but parent context's
    /// position is not updated.
    pub fn update_last_position(
        &self,
        identifier: &Identifier,
        position: usize,
    ) -> Result<bool, ContextError> {
        let found = self.update_position_if_found(identifier, position)?;

        if found {
            Ok(true)
        } else if let Some(parent) = &self.parent {
            let found_in_ancestor = parent.update_last_position(identifier, position)?;

            if !found_in_ancestor {
                let mut associations = self.associations.write()?;

                log::trace!("Updating {identifier}'s last position to {position:?}");

                associations.insert(identifier.clone(), (ContextData::Reserved, position));
            }

            Ok(false)
        } else {
            Ok(false)
        }
    }

    fn update_position_if_found(
        &self,
        identifier: &Identifier,
        position: usize,
    ) -> Result<bool, ContextError> {
        let mut associations = self.associations.write()?;

        if let Some((_, last_position)) = associations.get_mut(identifier) {
            log::trace!("Updating {identifier}'s last position to {position:?}");

            *last_position = position;

            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// Recovers the context from a poisoned state by recovering data from an error.
    ///
    /// This method is not used.
    pub fn _recover_from_poison(&mut self, error: &ContextError) {
        log::debug!("Context is recovering from poison error");

        let ContextError::PoisonErrorRecovered(recovered) = error;

        let mut new_associations = HashMap::new();

        for (identifier, (context_data, position)) in recovered.as_ref() {
            new_associations.insert(identifier.clone(), (context_data.clone(), *position));
        }

        self.associations = Arc::new(RwLock::new(new_associations));
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
    Reserved,
}

#[derive(Debug, Clone)]
pub enum ContextError {
    PoisonErrorRecovered(Arc<Associations>),
}

impl From<PoisonError<RwLockWriteGuard<'_, Associations>>> for ContextError {
    fn from(error: PoisonError<RwLockWriteGuard<'_, Associations>>) -> Self {
        let associations = error.into_inner().clone();

        Self::PoisonErrorRecovered(Arc::new(associations))
    }
}

impl From<PoisonError<RwLockReadGuard<'_, Associations>>> for ContextError {
    fn from(error: PoisonError<RwLockReadGuard<'_, Associations>>) -> Self {
        let associations = error.into_inner().clone();

        Self::PoisonErrorRecovered(Arc::new(associations))
    }
}

impl PartialEq for ContextError {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::PoisonErrorRecovered(left), Self::PoisonErrorRecovered(right)) => {
                Arc::ptr_eq(left, right)
            }
        }
    }
}

impl Display for ContextError {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Self::PoisonErrorRecovered(associations) => {
                write!(
                    f,
                    "Context poisoned with {} associations recovered",
                    associations.len()
                )
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{parse, Vm};

    use super::*;

    #[test]
    fn context_removes_variables() {
        let source = "
            let x = 5;
            let y = 10;
            let z = x + y;
            z
        ";
        let ast = parse(source).unwrap();
        let context = ast.context.clone();

        assert_eq!(Vm.run(ast), Ok(Some(Value::integer(15))));
        assert_eq!(context.association_count().unwrap(), 0);
    }

    #[test]
    fn garbage_collector_does_not_break_loops() {
        let source = "
            let mut z = 0;
            while z < 10 {
                z += 1;
            }
        ";
        let ast = parse(source).unwrap();
        let context = ast.context.clone();

        assert_eq!(Vm.run(ast), Ok(None));
        assert_eq!(context.association_count().unwrap(), 0);
    }
}
