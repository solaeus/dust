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
    is_clean: bool,
}

impl Context {
    pub fn new() -> Self {
        Self {
            variables: Arc::new(RwLock::new(BTreeMap::new())),
            is_clean: true,
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
        if let Some((ValueData::Value(value), usage_data)) = self.variables.read()?.get(identifier)
        {
            log::trace!("Using {identifier}'s value.");

            usage_data.inner().write()?.actual += 1;

            Ok(Some(value.clone()))
        } else {
            Ok(None)
        }
    }

    pub fn get_value(&self, identifier: &Identifier) -> Result<Option<Value>, RwLockPoisonError> {
        if let Some((ValueData::Value(value), _)) = self.variables.read()?.get(identifier) {
            log::trace!("Getting {identifier}'s value.");

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

    pub fn clean(&mut self) -> Result<(), RwLockPoisonError> {
        if self.is_clean {
            return Ok(());
        }

        self.variables
            .write()?
            .retain(|identifier, (value_data, usage_data)| {
                if let ValueData::Value(_) = value_data {
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

        self.is_clean = true;

        Ok(())
    }

    pub fn add_expected_use(&self, identifier: &Identifier) -> Result<bool, RwLockPoisonError> {
        if let Some((_, usage_data)) = self.variables.read()?.get(identifier) {
            log::trace!("Adding expected use for variable {identifier}.");

            usage_data.inner().write()?.expected += 1;

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
