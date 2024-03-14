use std::{
    collections::BTreeMap,
    sync::{Arc, RwLock},
};

use crate::{
    abstract_tree::{Identifier, Type},
    error::RwLockPoisonError,
    value::{BuiltInFunction, ValueInner},
    Value,
};

pub struct Context {
    inner: Arc<RwLock<BTreeMap<Identifier, (ValueData, UsageData)>>>,
}

#[derive(Clone, Debug)]
pub struct UsageData(Arc<RwLock<UsageDataInner>>);

#[derive(Clone, Debug)]
pub struct UsageDataInner {
    pub allowances: usize,
    pub uses: usize,
}

impl Default for UsageData {
    fn default() -> Self {
        UsageData(Arc::new(RwLock::new(UsageDataInner {
            allowances: 0,
            uses: 0,
        })))
    }
}

#[derive(Clone, Debug)]
pub enum ValueData {
    Type(Type),
    Value(Value),
}

impl Context {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(RwLock::new(BTreeMap::new())),
        }
    }

    pub fn with_data(data: BTreeMap<Identifier, (ValueData, UsageData)>) -> Self {
        Self {
            inner: Arc::new(RwLock::new(data)),
        }
    }

    pub fn inherit_types_from(other: &Context) -> Result<Self, RwLockPoisonError> {
        let mut new_data = BTreeMap::new();

        for (identifier, (value_data, usage_data)) in other.inner.read()?.iter() {
            if let ValueData::Type(r#type) = value_data {
                if let Type::Function { .. } = r#type {
                    new_data.insert(identifier.clone(), (value_data.clone(), usage_data.clone()));
                }
            }
        }

        Ok(Self::with_data(new_data))
    }

    pub fn inherit_data_from(other: &Context) -> Result<Self, RwLockPoisonError> {
        let mut new_data = BTreeMap::new();

        for (identifier, (value_data, usage_data)) in other.inner.read()?.iter() {
            if let ValueData::Type(r#type) = value_data {
                if let Type::Function { .. } = r#type {
                    new_data.insert(identifier.clone(), (value_data.clone(), usage_data.clone()));
                }
            }
            if let ValueData::Value(value) = value_data {
                if let ValueInner::Function { .. } = value.inner().as_ref() {
                    new_data.insert(identifier.clone(), (value_data.clone(), usage_data.clone()));
                }
            }
        }

        Ok(Self::with_data(new_data))
    }

    pub fn add_allowance(&self, identifier: &Identifier) -> Result<bool, RwLockPoisonError> {
        if let Some((_, usage_data)) = self.inner.read()?.get(identifier) {
            usage_data.0.write()?.allowances += 1;

            Ok(true)
        } else {
            Ok(false)
        }
    }

    pub fn use_data(
        &self,
        identifier: &Identifier,
    ) -> Result<Option<ValueData>, RwLockPoisonError> {
        let should_remove =
            if let Some((value_data, usage_data)) = self.inner.read()?.get(identifier) {
                let mut usage_data = usage_data.0.write()?;

                log::trace!("Adding use for variable: {identifier}");

                usage_data.uses += 1;

                if usage_data.uses == usage_data.allowances {
                    true
                } else {
                    return Ok(Some(value_data.clone()));
                }
            } else {
                false
            };

        if should_remove {
            log::trace!("Removing varialble: {identifier}");

            self.inner.write()?.remove(identifier);
        }

        let value_data = match identifier.as_str() {
            "output" => ValueData::Value(BuiltInFunction::output()),
            _ => return Ok(None),
        };

        Ok(Some(value_data))
    }

    pub fn use_type(&self, identifier: &Identifier) -> Result<Option<Type>, RwLockPoisonError> {
        if let Some((ValueData::Type(r#type), usage_data)) = self.inner.read()?.get(identifier) {
            log::trace!("Adding use for variable: {identifier}");

            usage_data.0.write()?.uses += 1;

            return Ok(Some(r#type.clone()));
        }

        let r#type = match identifier.as_str() {
            "output" => BuiltInFunction::Output.r#type(),
            _ => return Ok(None),
        };

        Ok(Some(r#type))
    }

    pub fn use_value(&self, identifier: &Identifier) -> Result<Option<Value>, RwLockPoisonError> {
        let should_remove = if let Some((ValueData::Value(value), usage_data)) =
            self.inner.read()?.get(identifier)
        {
            let mut usage_data = usage_data.0.write()?;

            log::trace!("Adding use for variable: {identifier}");

            usage_data.uses += 1;

            if usage_data.uses == usage_data.allowances {
                true
            } else {
                return Ok(Some(value.clone()));
            }
        } else {
            false
        };

        if should_remove {
            log::trace!("Removing varialble: {identifier}");

            self.inner.write()?.remove(identifier);
        }

        let value = match identifier.as_str() {
            "output" => BuiltInFunction::output(),
            _ => return Ok(None),
        };

        Ok(Some(value))
    }

    pub fn set_type(&self, identifier: Identifier, r#type: Type) -> Result<(), RwLockPoisonError> {
        self.inner
            .write()?
            .insert(identifier, (ValueData::Type(r#type), UsageData::default()));

        Ok(())
    }

    pub fn set_value(&self, identifier: Identifier, value: Value) -> Result<(), RwLockPoisonError> {
        let mut inner = self.inner.write()?;

        if let Some((_value_data, usage_data)) = inner.remove(&identifier) {
            inner.insert(identifier, (ValueData::Value(value), usage_data));
        } else {
            inner.insert(identifier, (ValueData::Value(value), UsageData::default()));
        }

        Ok(())
    }
}
