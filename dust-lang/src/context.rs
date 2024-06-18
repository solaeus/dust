use std::{
    collections::HashMap,
    sync::{Arc, RwLock, RwLockReadGuard},
};

use crate::{
    abstract_tree::Type,
    error::{RwLockPoisonError, ValidationError},
    identifier::Identifier,
    Value,
};

#[derive(Clone, Debug)]
pub struct Context<'a> {
    variables: Arc<RwLock<HashMap<Identifier, (VariableData, UsageData)>>>,
    parent: Option<&'a Context<'a>>,
    is_clean: Arc<RwLock<bool>>,
}

impl<'a> Context<'a> {
    pub fn new(parent: Option<&'a Context>) -> Self {
        Self {
            variables: Arc::new(RwLock::new(HashMap::new())),
            parent,
            is_clean: Arc::new(RwLock::new(true)),
        }
    }

    pub fn create_child<'b>(&'b self) -> Context<'b> {
        Context::new(Some(self))
    }

    pub fn inner(
        &self,
    ) -> Result<RwLockReadGuard<HashMap<Identifier, (VariableData, UsageData)>>, RwLockPoisonError>
    {
        Ok(self.variables.read()?)
    }

    pub fn contains(&self, identifier: &Identifier) -> Result<bool, RwLockPoisonError> {
        log::trace!("Checking that {identifier} exists.");

        if self.variables.read()?.contains_key(identifier) {
            Ok(true)
        } else if let Some(parent) = self.parent {
            parent.contains(identifier)
        } else {
            Ok(false)
        }
    }

    pub fn get_type(&self, identifier: &Identifier) -> Result<Option<Type>, ValidationError> {
        if let Some((data, _)) = self.variables.read()?.get(identifier) {
            log::trace!("Getting {identifier}'s type.");

            let r#type = match data {
                VariableData::Type(r#type) => r#type.clone(),
                VariableData::Value(value) => value.r#type(self)?,
            };

            Ok(Some(r#type.clone()))
        } else if let Some(parent) = self.parent {
            parent.get_type(identifier)
        } else {
            Ok(None)
        }
    }

    pub fn use_value(&self, identifier: &Identifier) -> Result<Option<Value>, RwLockPoisonError> {
        if let Some((VariableData::Value(value), usage_data)) =
            self.variables.read()?.get(identifier)
        {
            log::trace!("Using {identifier}'s value.");

            usage_data.inner().write()?.actual += 1;
            *self.is_clean.write()? = false;

            Ok(Some(value.clone()))
        } else if let Some(parent) = self.parent {
            parent.use_value(identifier)
        } else {
            Ok(None)
        }
    }

    pub fn get_value(&self, identifier: &Identifier) -> Result<Option<Value>, RwLockPoisonError> {
        if let Some((VariableData::Value(value), _)) = self.variables.read()?.get(identifier) {
            log::trace!("Getting {identifier}'s value.");

            Ok(Some(value.clone()))
        } else if let Some(parent) = self.parent {
            parent.get_value(identifier)
        } else {
            Ok(None)
        }
    }

    pub fn get_data(
        &self,
        identifier: &Identifier,
    ) -> Result<Option<(VariableData, UsageData)>, RwLockPoisonError> {
        if let Some(full_data) = self.variables.read()?.get(identifier) {
            log::trace!("Getting {identifier}'s value.");

            Ok(Some(full_data.clone()))
        } else if let Some(parent) = self.parent {
            parent.get_data(identifier)
        } else {
            Ok(None)
        }
    }

    pub fn set_type(&self, identifier: Identifier, r#type: Type) -> Result<(), RwLockPoisonError> {
        log::debug!("Setting {identifier} to type {}.", r#type);

        self.variables
            .write()?
            .insert(identifier, (VariableData::Type(r#type), UsageData::new()));

        Ok(())
    }

    pub fn set_value(
        &mut self,
        identifier: Identifier,
        value: Value,
    ) -> Result<(), RwLockPoisonError> {
        log::debug!("Setting {identifier} to value {value}.");

        let mut variables = self.variables.write()?;
        let old_usage_data = variables
            .remove(&identifier)
            .map(|(_, usage_data)| usage_data);

        if let Some(usage_data) = old_usage_data {
            variables.insert(identifier, (VariableData::Value(value), usage_data));
        } else {
            variables.insert(identifier, (VariableData::Value(value), UsageData::new()));
        }

        Ok(())
    }

    pub fn clean(&mut self) -> Result<(), RwLockPoisonError> {
        if *self.is_clean.read()? {
            return Ok(());
        }

        self.variables
            .write()?
            .retain(|identifier, (value_data, usage_data)| {
                if let VariableData::Value(_) = value_data {
                    let usage = usage_data.inner().read().unwrap();

                    if usage.actual < usage.expected {
                        true
                    } else {
                        log::trace!("Removing variable {identifier}.");

                        false
                    }
                } else {
                    false
                }
            });

        *self.is_clean.write()? = true;

        Ok(())
    }

    pub fn is_clean(&mut self) -> Result<bool, RwLockPoisonError> {
        if *self.is_clean.read()? {
            Ok(true)
        } else {
            for (_, (_, usage_data)) in self.variables.read()?.iter() {
                let usage_data = usage_data.inner().read().unwrap();

                if usage_data.actual > usage_data.expected {
                    return Ok(false);
                }
            }

            Ok(true)
        }
    }

    pub fn add_expected_use(&self, identifier: &Identifier) -> Result<bool, RwLockPoisonError> {
        if let Some((_, usage_data)) = self.variables.read()?.get(identifier) {
            log::trace!("Adding expected use for variable {identifier}.");

            usage_data.inner().write()?.expected += 1;

            Ok(true)
        } else if let Some(parent) = self.parent {
            parent.add_expected_use(identifier)
        } else {
            Ok(false)
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum VariableData {
    Type(Type),
    Value(Value),
}

#[derive(Clone, Debug)]
pub struct UsageData(Arc<RwLock<UsageDataInner>>);

impl UsageData {
    pub fn inner(&self) -> &Arc<RwLock<UsageDataInner>> {
        &self.0
    }
}

#[derive(Clone, Debug)]
pub struct UsageDataInner {
    pub actual: u32,
    pub expected: u32,
}

impl UsageData {
    pub fn new() -> Self {
        UsageData(Arc::new(RwLock::new(UsageDataInner {
            actual: 0,
            expected: 0,
        })))
    }
}
