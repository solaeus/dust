use std::{
    collections::BTreeMap,
    sync::{Arc, RwLock},
};

use crate::{abstract_tree::Identifier, error::RwLockPoisonError, Value};

pub struct Context {
    inner: Arc<RwLock<BTreeMap<Identifier, Value>>>,
}

impl Context {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(RwLock::new(BTreeMap::new())),
        }
    }

    pub fn with_values(values: BTreeMap<Identifier, Value>) -> Self {
        Self {
            inner: Arc::new(RwLock::new(values)),
        }
    }

    pub fn get(&self, identifier: &Identifier) -> Result<Option<Value>, RwLockPoisonError> {
        let value = self.inner.read()?.get(identifier).cloned();

        Ok(value)
    }

    pub fn set(&self, identifier: Identifier, value: Value) -> Result<(), RwLockPoisonError> {
        self.inner.write()?.insert(identifier, value);

        Ok(())
    }
}
