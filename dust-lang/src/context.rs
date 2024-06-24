use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};

use crate::{
    abstract_tree::Type,
    error::{PoisonError, ValidationError},
    identifier::Identifier,
    Value,
};

#[derive(Clone, Debug)]
pub struct Context {
    data: Arc<RwLock<ContextData>>,
    is_clean: Arc<RwLock<bool>>,
}

#[derive(Clone, Debug)]
struct ContextData {
    variables: HashMap<Identifier, (VariableData, UsageData)>,
    parent: Option<Context>,
}

impl Context {
    pub fn new(parent: Option<Context>) -> Self {
        Context {
            data: Arc::new(RwLock::new(ContextData {
                variables: HashMap::new(),
                parent,
            })),
            is_clean: Arc::new(RwLock::new(true)),
        }
    }

    pub fn create_child(&self) -> Context {
        Context::new(Some(self.clone()))
    }

    pub fn contains(&self, identifier: &Identifier) -> Result<bool, PoisonError> {
        log::trace!("Checking that {identifier} exists.");

        let data = self.data.read()?;

        if data.variables.contains_key(identifier) {
            Ok(true)
        } else if let Some(parent) = &data.parent {
            parent.contains(identifier)
        } else {
            Ok(false)
        }
    }

    pub fn get_type(&self, identifier: &Identifier) -> Result<Option<Type>, ValidationError> {
        log::trace!("Getting {identifier}'s type.");

        let data = self.data.read()?;

        if let Some((data, _)) = data.variables.get(identifier) {
            let r#type = match data {
                VariableData::Type(r#type) => r#type.clone(),
                VariableData::Value(value) => value.r#type(self)?,
            };

            Ok(Some(r#type.clone()))
        } else if let Some(parent) = &data.parent {
            parent.get_type(identifier)
        } else {
            Ok(None)
        }
    }

    pub fn use_value(&self, identifier: &Identifier) -> Result<Option<Value>, PoisonError> {
        log::trace!("Using {identifier}'s value.");

        let data = self.data.read()?;

        if let Some((VariableData::Value(value), usage_data)) = data.variables.get(identifier) {
            usage_data.inner().write()?.actual += 1;
            *self.is_clean.write()? = false;

            Ok(Some(value.clone()))
        } else if let Some(parent) = &data.parent {
            parent.use_value(identifier)
        } else {
            Ok(None)
        }
    }

    pub fn get_value(&self, identifier: &Identifier) -> Result<Option<Value>, PoisonError> {
        log::trace!("Getting {identifier}'s value.");

        let data = self.data.read()?;

        if let Some((VariableData::Value(value), _)) = data.variables.get(identifier) {
            Ok(Some(value.clone()))
        } else if let Some(parent) = &data.parent {
            parent.get_value(identifier)
        } else {
            Ok(None)
        }
    }

    pub fn set_type(&self, identifier: Identifier, r#type: Type) -> Result<(), PoisonError> {
        log::debug!("Setting {identifier} to type {}.", r#type);

        self.data
            .write()?
            .variables
            .insert(identifier, (VariableData::Type(r#type), UsageData::new()));

        Ok(())
    }

    pub fn set_value(&self, identifier: Identifier, value: Value) -> Result<(), PoisonError> {
        log::debug!("Setting {identifier} to value {value}.");

        let mut data = self.data.write()?;
        let usage_data = data
            .variables
            .remove(&identifier)
            .map(|(_, usage_data)| usage_data)
            .unwrap_or(UsageData::new());

        data.variables
            .insert(identifier, (VariableData::Value(value), usage_data));

        Ok(())
    }

    pub fn add_expected_use(&self, identifier: &Identifier) -> Result<bool, PoisonError> {
        let data = self.data.read()?;

        if let Some((_, usage_data)) = data.variables.get(identifier) {
            log::trace!("Adding expected use for variable {identifier}.");

            usage_data.inner().write()?.expected += 1;

            Ok(true)
        } else if let Some(parent) = &data.parent {
            parent.add_expected_use(identifier)
        } else {
            Ok(false)
        }
    }

    pub fn clean(&self) -> Result<(), PoisonError> {
        if *self.is_clean.read()? {
            return Ok(());
        }

        self.data.write()?.variables.retain(
            |identifier, (value_data, usage_data)| match value_data {
                VariableData::Type(_) => true,
                VariableData::Value(_) => {
                    let usage = usage_data.inner().read().unwrap();

                    if usage.actual < usage.expected {
                        true
                    } else {
                        log::trace!("Removing {identifier}.");

                        false
                    }
                }
            },
        );

        *self.is_clean.write()? = true;

        Ok(())
    }

    pub fn is_clean(&mut self) -> Result<bool, PoisonError> {
        Ok(*self.is_clean.read()?)
    }
}

impl Default for Context {
    fn default() -> Self {
        Context::new(None)
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
