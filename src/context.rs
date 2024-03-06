use std::{
    collections::BTreeMap,
    sync::{Arc, RwLock},
};

use crate::{
    abstract_tree::{Identifier, Type},
    error::RwLockPoisonError,
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

    pub fn get_data(
        &self,
        identifier: &Identifier,
    ) -> Result<Option<ValueData>, RwLockPoisonError> {
        Ok(self.inner.read()?.get(identifier).cloned())
    }

    pub fn get_type(&self, identifier: &Identifier) -> Result<Option<Type>, RwLockPoisonError> {
        if let Some(ValueData::Type(r#type)) = self.inner.read()?.get(identifier) {
            Ok(Some(r#type.clone()))
        } else {
            Ok(None)
        }
    }

    pub fn get_value(&self, identifier: &Identifier) -> Result<Option<Value>, RwLockPoisonError> {
        if let Some(ValueData::Value(value)) = self.inner.read()?.get(identifier) {
            Ok(Some(value.clone()))
        } else {
            Ok(None)
        }
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
