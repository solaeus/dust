use std::{
    collections::BTreeMap,
    sync::{Arc, RwLock, RwLockReadGuard},
};

use crate::{
    abstract_tree::Type,
    error::{RwLockPoisonError, ValidationError},
    identifier::Identifier,
    Value,
};

#[derive(Clone, Debug)]
pub struct Context {
    variables: Arc<RwLock<BTreeMap<Identifier, (ValueData, UsageData)>>>,
}

impl Context {
    pub fn new() -> Self {
        Self {
            variables: Arc::new(RwLock::new(BTreeMap::new())),
        }
    }

    pub fn with_data(data: BTreeMap<Identifier, (ValueData, UsageData)>) -> Self {
        Self {
            variables: Arc::new(RwLock::new(data)),
        }
    }

    pub fn inner(
        &self,
    ) -> Result<RwLockReadGuard<BTreeMap<Identifier, (ValueData, UsageData)>>, RwLockPoisonError>
    {
        Ok(self.variables.read()?)
    }

    pub fn inherit_types_from(&self, other: &Context) -> Result<(), RwLockPoisonError> {
        let mut self_data = self.variables.write()?;

        for (identifier, (value_data, usage_data)) in other.variables.read()?.iter() {
            if let ValueData::Type(Type::Function { .. }) = value_data {
                log::trace!("Inheriting type of variable {identifier}.");

                self_data.insert(identifier.clone(), (value_data.clone(), usage_data.clone()));
            }
        }

        Ok(())
    }

    pub fn inherit_data_from(&self, other: &Context) -> Result<(), RwLockPoisonError> {
        let mut self_data = self.variables.write()?;

        for (identifier, (value_data, usage_data)) in other.variables.read()?.iter() {
            log::trace!("Inheriting variable {identifier}.");

            self_data.insert(identifier.clone(), (value_data.clone(), usage_data.clone()));
        }

        Ok(())
    }

    pub fn contains(&self, identifier: &Identifier) -> Result<bool, RwLockPoisonError> {
        log::trace!("Checking that {identifier} exists.");

        Ok(self.variables.read()?.contains_key(identifier))
    }

    pub fn get_type(&self, identifier: &Identifier) -> Result<Option<Type>, ValidationError> {
        if let Some((value_data, _)) = self.variables.read()?.get(identifier) {
            log::trace!("Using {identifier}'s type.");

            let r#type = match value_data {
                ValueData::Type(r#type) => r#type.clone(),
                ValueData::Value(value) => value.r#type(self)?,
            };

            Ok(Some(r#type.clone()))
        } else {
            Ok(None)
        }
    }

    pub fn use_value(&self, identifier: &Identifier) -> Result<Option<Value>, RwLockPoisonError> {
        if let Some((ValueData::Value(value), usage_data)) =
            self.variables.write()?.get_mut(identifier)
        {
            log::trace!("Using {identifier}'s value.");

            usage_data.actual += 1;

            Ok(Some(value.clone()))
        } else {
            Ok(None)
        }
    }

    pub fn set_type(&self, identifier: Identifier, r#type: Type) -> Result<(), RwLockPoisonError> {
        log::debug!("Setting {identifier} to type {}.", r#type);

        self.variables
            .write()?
            .insert(identifier, (ValueData::Type(r#type), UsageData::new()));

        Ok(())
    }

    pub fn set_value(&self, identifier: Identifier, value: Value) -> Result<(), RwLockPoisonError> {
        log::debug!("Setting {identifier} to value {value}.");

        let mut variables = self.variables.write()?;
        let old_usage_data = variables
            .remove(&identifier)
            .map(|(_, usage_data)| usage_data);

        if let Some(usage_data) = old_usage_data {
            variables.insert(identifier, (ValueData::Value(value), usage_data));
        } else {
            variables.insert(identifier, (ValueData::Value(value), UsageData::new()));
        }

        Ok(())
    }

    pub fn remove(&self, identifier: &Identifier) -> Result<Option<ValueData>, RwLockPoisonError> {
        let removed = self
            .variables
            .write()?
            .remove(identifier)
            .map(|(value_data, _)| value_data);

        Ok(removed)
    }

    pub fn clean(&mut self) -> Result<(), RwLockPoisonError> {
        self.variables
            .write()?
            .retain(|identifier, (_, usage_data)| {
                if usage_data.actual < usage_data.expected {
                    true
                } else {
                    log::trace!("Removing variable {identifier}.");

                    false
                }
            });

        Ok(())
    }

    pub fn add_expected_use(&self, identifier: &Identifier) -> Result<bool, RwLockPoisonError> {
        let mut variables = self.variables.write()?;

        if let Some((_, usage_data)) = variables.get_mut(identifier) {
            log::trace!("Adding expected use for variable {identifier}.");

            usage_data.expected += 1;

            Ok(true)
        } else {
            Ok(false)
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum ValueData {
    Type(Type),
    Value(Value),
}

#[derive(Clone, Debug)]
pub struct UsageData {
    pub actual: u32,
    pub expected: u32,
}

impl UsageData {
    pub fn new() -> Self {
        Self {
            actual: 0,
            expected: 0,
        }
    }
}
