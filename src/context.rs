use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};

use crate::{error::rw_lock_error::RwLockError, Type, Value};

#[derive(Clone)]
pub enum ValueData {
    Value {
        inner: Value,
        runtime_uses: Arc<RwLock<u16>>,
    },
    ExpectedType {
        inner: Type,
    },
}

pub struct Context {
    inner: Arc<RwLock<HashMap<String, ValueData>>>,
}

impl Context {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn inherit_from(other: &Context) -> Result<Context, RwLockError> {
        let mut new_variables = HashMap::new();

        for (identifier, value_data) in other.inner.read()?.iter() {
            new_variables.insert(identifier.clone(), value_data.clone());
        }

        Ok(Context {
            inner: Arc::new(RwLock::new(new_variables)),
        })
    }

    pub fn get_value(&self, key: &str) -> Result<Option<Value>, RwLockError> {
        if let Some(value_data) = self.inner.read()?.get(key) {
            if let ValueData::Value { inner, .. } = value_data {
                Ok(Some(inner.clone()))
            } else {
                Ok(None)
            }
        } else {
            Ok(None)
        }
    }

    pub fn get_type(&self, key: &str) -> Result<Option<Type>, RwLockError> {
        if let Some(value_data) = self.inner.read()?.get(key) {
            match value_data {
                ValueData::Value { inner, .. } => Ok(Some(inner.r#type())),
                ValueData::ExpectedType { inner, .. } => Ok(Some(inner.clone())),
            }
        } else {
            Ok(None)
        }
    }

    pub fn set_value(&self, key: String, value: Value) -> Result<(), RwLockError> {
        self.inner.write()?.insert(
            key,
            ValueData::Value {
                inner: value,
                runtime_uses: Arc::new(RwLock::new(0)),
            },
        );

        Ok(())
    }

    pub fn set_type(&self, key: String, r#type: Type) -> Result<(), RwLockError> {
        self.inner
            .write()?
            .insert(key, ValueData::ExpectedType { inner: r#type });

        Ok(())
    }
}
