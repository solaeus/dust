use std::{
    collections::HashMap,
    fmt::{self, Debug, Display, Formatter},
    sync::{Arc, RwLock, RwLockReadGuard},
};

use serde::{Deserialize, Serialize};

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

impl Eq for ValueData {}

impl PartialEq for ValueData {
    fn eq(&self, other: &Self) -> bool {
        todo!()
    }
}

impl PartialOrd for ValueData {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for ValueData {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        todo!()
    }
}

#[derive(Clone)]
pub struct Context {
    inner: Arc<RwLock<HashMap<String, ValueData>>>,
}

impl Context {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn inner(&self) -> Result<RwLockReadGuard<HashMap<String, ValueData>>, RwLockError> {
        Ok(self.inner.read()?)
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

impl Eq for Context {}

impl PartialEq for Context {
    fn eq(&self, other: &Self) -> bool {
        let self_variables = self.inner().unwrap();
        let other_variables = other.inner().unwrap();

        if self_variables.len() != other_variables.len() {
            return false;
        }

        for ((left_key, left_value_data), (right_key, right_value_data)) in
            self_variables.iter().zip(other_variables.iter())
        {
            if left_key != right_key || left_value_data != right_value_data {
                return false;
            }
        }

        true
    }
}

impl PartialOrd for Context {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Context {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        todo!()
    }
}

impl<'de> Deserialize<'de> for Context {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        todo!()
    }
}

impl Serialize for Context {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        todo!()
    }
}

impl Debug for Context {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{self}")
    }
}

impl Display for Context {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        todo!()
    }
}
