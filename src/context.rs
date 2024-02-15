use std::{
    cmp::Ordering,
    collections::BTreeMap,
    sync::{Arc, RwLock, RwLockReadGuard},
};

use crate::{
    built_in_type_definitions::all_built_in_type_definitions, built_in_values::all_built_in_values,
    error::rw_lock_error::RwLockError, Type, TypeDefinition, Value,
};

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
            let existing_data = self_variables.get(identifier);

            if let Some(ValueData::Value { .. }) = existing_data {
                continue;
            } else {
                self_variables.insert(identifier.clone(), value_data.clone());
            }
        }

        Ok(())
    }

    pub fn get_value(&self, key: &str) -> Result<Option<Value>, RwLockError> {
        if let Some(value_data) = self.inner.read()?.get(key) {
            if let ValueData::Value { inner, .. } = value_data {
                return Ok(Some(inner.clone()));
            }
        }

        for built_in_value in all_built_in_values() {
            if key == built_in_value.name() {
                return Ok(Some(built_in_value.get().clone()));
            }
        }

        Ok(None)
    }

    pub fn get_type(&self, key: &str) -> Result<Option<Type>, RwLockError> {
        if let Some(value_data) = self.inner.read()?.get(key) {
            match value_data {
                ValueData::Value { inner, .. } => Ok(Some(inner.r#type())),
                ValueData::ExpectedType { inner, .. } => Ok(Some(inner.clone())),
                ValueData::TypeDefinition(_) => todo!(),
            }
        } else {
            Ok(None)
        }
    }

    pub fn get_definition(&self, key: &str) -> Result<Option<TypeDefinition>, RwLockError> {
        if let Some(value_data) = self.inner.read()?.get(key) {
            if let ValueData::TypeDefinition(definition) = value_data {
                return Ok(Some(definition.clone()));
            }
        }

        for built_in_definition in all_built_in_type_definitions() {
            if key == built_in_definition.name() {
                return Ok(Some(built_in_definition.get().clone()));
            }
        }

        Ok(None)
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

    pub fn set_definition(
        &self,
        key: String,
        definition: TypeDefinition,
    ) -> Result<(), RwLockError> {
        self.inner
            .write()?
            .insert(key, ValueData::TypeDefinition(definition));

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
    TypeDefinition(TypeDefinition),
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
            (ValueData::TypeDefinition(left), ValueData::TypeDefinition(right)) => left.cmp(right),
            (ValueData::TypeDefinition(_), _) => Greater,
            (ValueData::ExpectedType { .. }, _) => Less,
        }
    }
}
