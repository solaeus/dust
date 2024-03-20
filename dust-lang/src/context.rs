use std::{
    collections::BTreeMap,
    sync::{Arc, RwLock, RwLockReadGuard},
};

use crate::{
    abstract_tree::{Identifier, Type},
    error::{RwLockPoisonError, ValidationError},
    value::{BUILT_IN_FUNCTIONS, BUILT_IN_MODULES},
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

    pub fn inner(
        &self,
    ) -> Result<RwLockReadGuard<BTreeMap<Identifier, ValueData>>, RwLockPoisonError> {
        Ok(self.inner.read()?)
    }

    pub fn inherit_types_from(&self, other: &Context) -> Result<(), RwLockPoisonError> {
        let mut self_data = self.inner.write()?;

        for (identifier, value_data) in other.inner.read()?.iter() {
            if let ValueData::Type(Type::Function { .. }) = value_data {
                self_data.insert(identifier.clone(), value_data.clone());
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
            for module in BUILT_IN_MODULES {
                if identifier.as_str() == module.name() {
                    return Ok(true);
                }
            }

            for function in BUILT_IN_FUNCTIONS {
                if identifier.as_str() == function.name() {
                    return Ok(true);
                }
            }

            Ok(false)
        }
    }

    pub fn get_type(&self, identifier: &Identifier) -> Result<Option<Type>, ValidationError> {
        if let Some(value_data) = self.inner.read()?.get(identifier) {
            let r#type = match value_data {
                ValueData::Type(r#type) => {
                    if let Type::Named(name) = r#type {
                        return self.get_type(name);
                    }

                    r#type.clone()
                }
                ValueData::Value(value) => value.r#type(self)?,
            };

            return Ok(Some(r#type.clone()));
        }

        for module in BUILT_IN_MODULES {
            if identifier.as_str() == module.name() {
                return Ok(Some(module.r#type()));
            }
        }

        for function in BUILT_IN_MODULES {
            if identifier.as_str() == function.name() {
                return Ok(Some(function.r#type()));
            }
        }

        Ok(None)
    }

    pub fn get_value(&self, identifier: &Identifier) -> Result<Option<Value>, RwLockPoisonError> {
        if let Some(ValueData::Value(value)) = self.inner.read()?.get(identifier) {
            Ok(Some(value.clone()))
        } else {
            for module in BUILT_IN_MODULES {
                if identifier.as_str() == module.name() {
                    return Ok(Some(module.value()));
                }
            }

            for function in BUILT_IN_MODULES {
                if identifier.as_str() == function.name() {
                    return Ok(Some(function.value()));
                }
            }

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
