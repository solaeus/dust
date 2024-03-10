use std::{
    collections::BTreeMap,
    sync::{Arc, RwLock},
};

use crate::{
    abstract_tree::{Identifier, Type},
    error::RwLockPoisonError,
    value::BuiltInFunction,
    Value,
};

pub struct Context {
    inner: Arc<RwLock<BTreeMap<Identifier, ValueData>>>,
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

    pub fn with_data(data: BTreeMap<Identifier, ValueData>) -> Self {
        Self {
            inner: Arc::new(RwLock::new(data)),
        }
    }

    pub fn with_types_from(other: &Context) -> Result<Self, RwLockPoisonError> {
        let mut new_data = BTreeMap::new();

        for (identifier, value_data) in other.inner.read()?.iter() {
            if let ValueData::Type(_) = value_data {
                new_data.insert(identifier.clone(), value_data.clone());
            }
        }

        Ok(Self::with_data(new_data))
    }

    pub fn with_data_from(other: &Context) -> Result<Self, RwLockPoisonError> {
        let mut new_data = BTreeMap::new();

        for (identifier, value_data) in other.inner.read()?.iter() {
            new_data.insert(identifier.clone(), value_data.clone());
        }

        Ok(Self::with_data(new_data))
    }

    pub fn get_data(
        &self,
        identifier: &Identifier,
    ) -> Result<Option<ValueData>, RwLockPoisonError> {
        if let Some(value_data) = self.inner.read()?.get(identifier) {
            return Ok(Some(value_data.clone()));
        }

        let value_data = match identifier.as_str() {
            "output" => ValueData::Value(BuiltInFunction::output()),
            _ => return Ok(None),
        };

        Ok(Some(value_data))
    }

    pub fn get_type(&self, identifier: &Identifier) -> Result<Option<Type>, RwLockPoisonError> {
        if let Some(ValueData::Type(r#type)) = self.inner.read()?.get(identifier) {
            return Ok(Some(r#type.clone()));
        }

        let r#type = match identifier.as_str() {
            "output" => BuiltInFunction::Output.r#type(),
            _ => return Ok(None),
        };

        Ok(Some(r#type))
    }

    pub fn get_value(&self, identifier: &Identifier) -> Result<Option<Value>, RwLockPoisonError> {
        if let Some(ValueData::Value(value)) = self.inner.read()?.get(identifier) {
            return Ok(Some(value.clone()));
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
            .insert(identifier, ValueData::Type(r#type));

        Ok(())
    }

    pub fn set_value(&self, identifier: Identifier, value: Value) -> Result<(), RwLockPoisonError> {
        self.inner
            .write()?
            .insert(identifier, ValueData::Value(value));

        Ok(())
    }
}
