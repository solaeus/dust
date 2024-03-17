use std::{
    collections::BTreeMap,
    sync::{Arc, RwLock},
};

use crate::{
    abstract_tree::{Identifier, Type},
    error::RwLockPoisonError,
    value::{BuiltInFunction, BuiltInValue},
    Value,
};

#[derive(Clone, Debug)]
pub struct Context {
    inner: Arc<RwLock<BTreeMap<Identifier, ValueData>>>,
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

    pub fn inherit_types_from(&self, other: &Context) -> Result<(), RwLockPoisonError> {
        let mut self_data = self.inner.write()?;

        for (identifier, value_data) in other.inner.read()?.iter() {
            if let ValueData::Type(r#type) = value_data {
                if let Type::Function { .. } = r#type {
                    self_data.insert(identifier.clone(), value_data.clone());
                }
            }
        }

        Ok(())
    }

    pub fn inherit_data_from(&self, other: &Context) -> Result<(), RwLockPoisonError> {
        let mut self_data = self.inner.write()?;

        for (identifier, value_data) in other.inner.read()?.iter() {
            self_data.insert(identifier.clone(), value_data.clone());
        }

        Ok(())
    }

    pub fn contains(&self, identifier: &Identifier) -> Result<bool, RwLockPoisonError> {
        if self.inner.read()?.contains_key(identifier) {
            Ok(true)
        } else {
            match identifier.as_str() {
                "io" | "output" => Ok(true),
                _ => Ok(false),
            }
        }
    }

    pub fn get_type(&self, identifier: &Identifier) -> Result<Option<Type>, RwLockPoisonError> {
        if let Some(value_data) = self.inner.read()?.get(identifier) {
            let r#type = match value_data {
                ValueData::Type(r#type) => r#type.clone(),
                ValueData::Value(value) => value.r#type(),
            };

            return Ok(Some(r#type.clone()));
        }

        let r#type = match identifier.as_str() {
            "io" => BuiltInValue::Io.r#type(),
            "output" => BuiltInFunction::Output.r#type(),
            _ => return Ok(None),
        };

        Ok(Some(r#type))
    }

    pub fn get_value(&self, identifier: &Identifier) -> Result<Option<Value>, RwLockPoisonError> {
        if let Some(ValueData::Value(value)) = self.inner.read()?.get(identifier) {
            Ok(Some(value.clone()))
        } else {
            let value = match identifier.as_str() {
                "io" => BuiltInValue::Io.value(),
                "output" => Value::built_in_function(BuiltInFunction::Output),
                _ => return Ok(None),
            };

            Ok(Some(value))
        }
    }

    pub fn set_type(&self, identifier: Identifier, r#type: Type) -> Result<(), RwLockPoisonError> {
        self.inner
            .write()?
            .insert(identifier, ValueData::Type(r#type));

        Ok(())
    }

    pub fn set_value(&self, identifier: Identifier, value: Value) -> Result<(), RwLockPoisonError> {
        let mut inner = self.inner.write()?;

        inner.insert(identifier, ValueData::Value(value));

        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum ValueData {
    Type(Type),
    Value(Value),
}
