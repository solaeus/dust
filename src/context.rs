use std::{
    collections::BTreeMap,
    sync::{Arc, RwLock},
};

use crate::{error::RwLockPoisonError, Value};

pub struct Context {
    inner: Arc<RwLock<BTreeMap<String, Value>>>,
}

impl Context {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(RwLock::new(BTreeMap::new())),
        }
    }

    pub fn with_values(values: BTreeMap<String, Value>) -> Self {
        Self {
            inner: Arc::new(RwLock::new(values)),
        }
    }

    pub fn get(&self, key: &str) -> Result<Option<Value>, RwLockPoisonError> {
        let value = self.inner.read()?.get(key).cloned();

        Ok(value)
    }

    pub fn set(&self, key: String, value: Value) -> Result<(), RwLockPoisonError> {
        self.inner.write()?.insert(key, value);

        Ok(())
    }
}
