use std::{
    cmp::Ordering,
    collections::BTreeMap,
    sync::{Arc, RwLock, RwLockReadGuard},
};

use crate::{error::rw_lock_error::RwLockError, Type, Value};

#[derive(Clone, Debug)]
pub struct Context {
    inner: Arc<RwLock<BTreeMap<String, ValueData>>>,
}

impl Context {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(RwLock::new(BTreeMap::new())),
        }
    }

    pub fn inner(&self) -> Result<RwLockReadGuard<BTreeMap<String, ValueData>>, RwLockError> {
        Ok(self.inner.read()?)
    }

    pub fn with_variables_from(other: &Context) -> Result<Context, RwLockError> {
        let mut new_variables = BTreeMap::new();

        for (identifier, value_data) in other.inner.read()?.iter() {
            new_variables.insert(identifier.clone(), value_data.clone());
        }

        Ok(Context {
            inner: Arc::new(RwLock::new(new_variables)),
        })
    }

    pub fn inherit_from(&self, other: &Context) -> Result<(), RwLockError> {
        let mut self_variables = self.inner.write()?;

        for (identifier, value_data) in other.inner.read()?.iter() {
            self_variables.insert(identifier.clone(), value_data.clone());
        }

        Ok(())
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

    pub fn unset(&self, key: &str) -> Result<(), RwLockError> {
        self.inner.write()?.remove(key);

        Ok(())
    }
}

impl Default for Context {
    fn default() -> Self {
        Context::new()
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
    fn cmp(&self, other: &Self) -> Ordering {
        let left = self.inner().unwrap();
        let right = other.inner().unwrap();

        left.cmp(&right)
    }
}

#[derive(Clone, Debug)]
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
        match (self, other) {
            (
                ValueData::Value {
                    inner: left_inner,
                    runtime_uses: left_runtime_uses,
                },
                ValueData::Value {
                    inner: right_inner,
                    runtime_uses: right_runtime_uses,
                },
            ) => {
                if left_inner != right_inner {
                    return false;
                } else {
                    *left_runtime_uses.read().unwrap() == *right_runtime_uses.read().unwrap()
                }
            }
            (
                ValueData::ExpectedType { inner: left_inner },
                ValueData::ExpectedType { inner: right_inner },
            ) => left_inner == right_inner,
            _ => false,
        }
    }
}

impl PartialOrd for ValueData {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for ValueData {
    fn cmp(&self, other: &Self) -> Ordering {
        use Ordering::*;

        match (self, other) {
            (
                ValueData::Value {
                    inner: inner_left, ..
                },
                ValueData::Value {
                    inner: inner_right, ..
                },
            ) => inner_left.cmp(inner_right),
            (ValueData::Value { .. }, _) => Greater,
            (
                ValueData::ExpectedType { inner: inner_left },
                ValueData::ExpectedType { inner: inner_right },
            ) => inner_left.cmp(inner_right),
            (ValueData::ExpectedType { .. }, _) => Less,
        }
    }
}
