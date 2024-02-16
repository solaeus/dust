//! An execution context that stores variables and type data during the
//! [Interpreter][crate::Interpreter]'s abstraction and execution process.
//!
//! ## Setting values
//!
//! When data is stored in a context, it can be accessed by dust source code.
//! This allows you to insert values and type definitions before any code is
//! interpreted.
//!
//! ```
//! # use dust_lang::*;
//! let context = Context::new();
//!
//! context.set_value(
//!     "foobar".into(),
//!     Value::String("FOOBAR".to_string())
//! ).unwrap();
//!
//! interpret_with_context("output foobar", context);
//!
//! // Stdout: "FOOBAR"
//! ```
//!
//! ## Built-in values and type definitions
//!
//! When looking up values and definitions, the Context will try to use one that
//! has been explicitly set. If nothing is found, it will then check the built-
//! in values and type definitions for a match. This means that the user can
//! override the built-ins.
use std::{
    cmp::Ordering,
    collections::BTreeMap,
    sync::{Arc, RwLock, RwLockReadGuard},
};

use crate::{
    built_in_type_definitions::all_built_in_type_definitions, built_in_values::all_built_in_values,
    error::rw_lock_error::RwLockError, Identifier, Type, TypeDefinition, Value,
};

/// An execution context that variable and type data during the [Interpreter]'s
/// abstraction and execution process.
///
/// See the [module-level docs][self] for more info.
#[derive(Clone, Debug)]
pub struct Context {
    inner: Arc<RwLock<BTreeMap<Identifier, ValueData>>>,
}

impl Context {
    /// Return a new, empty Context.
    pub fn new() -> Self {
        Self {
            inner: Arc::new(RwLock::new(BTreeMap::new())),
        }
    }

    /// Return a lock guard to the inner BTreeMap.
    pub fn inner(&self) -> Result<RwLockReadGuard<BTreeMap<Identifier, ValueData>>, RwLockError> {
        Ok(self.inner.read()?)
    }

    /// Create a new context with all of the data from an existing context.
    pub fn with_variables_from(other: &Context) -> Result<Context, RwLockError> {
        let mut new_variables = BTreeMap::new();

        for (identifier, value_data) in other.inner.read()?.iter() {
            new_variables.insert(identifier.clone(), value_data.clone());
        }

        Ok(Context {
            inner: Arc::new(RwLock::new(new_variables)),
        })
    }

    /// Modify a context to take on all of the key-value pairs of another.
    ///
    /// In the case of the conflict, the inherited value will override the previous
    /// value.
    ///
    /// ```
    /// # use dust_lang::*;
    /// let first_context = Context::new();
    /// let second_context = Context::new();
    ///
    /// second_context.set_value(
    ///     "Foo".into(),
    ///     Value::String("Bar".to_string())
    /// );
    ///
    /// first_context.inherit_from(&second_context).unwrap();
    ///
    /// assert_eq!(first_context, second_context);
    /// ```
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

    /// Get a value from the context.
    ///
    /// This will also return a built-in value if one matches the key. See the
    /// [module-level docs][self] for more info.
    pub fn get_value(&self, identifier: &Identifier) -> Result<Option<Value>, RwLockError> {
        if let Some(value_data) = self.inner.read()?.get(identifier) {
            if let ValueData::Value { inner, .. } = value_data {
                return Ok(Some(inner.clone()));
            }
        }

        for built_in_value in all_built_in_values() {
            println!("{} {}", built_in_value.name(), identifier.inner());
            if built_in_value.name() == identifier.inner().as_ref() {
                return Ok(Some(built_in_value.get().clone()));
            }
        }

        Ok(None)
    }

    /// Get a type from the context.
    ///
    /// If the key matches a stored value, its type will be returned. It if
    /// matches a type hint, the type hint will be returned.
    pub fn get_type(&self, identifier: &Identifier) -> Result<Option<Type>, RwLockError> {
        if let Some(value_data) = self.inner.read()?.get(identifier) {
            match value_data {
                ValueData::Value { inner, .. } => return Ok(Some(inner.r#type())),
                ValueData::TypeHint { inner, .. } => return Ok(Some(inner.clone())),
                ValueData::TypeDefinition(_) => todo!(),
            }
        }

        for built_in_value in all_built_in_values() {
            if built_in_value.name() == identifier.inner().as_ref() {
                return Ok(Some(built_in_value.get().r#type()));
            }
        }

        Ok(None)
    }

    /// Get a type definition from the context.
    ///
    /// This will also return a built-in type definition if one matches the key.
    /// See the [module-level docs][self] for more info.
    pub fn get_definition(
        &self,
        identifier: &Identifier,
    ) -> Result<Option<TypeDefinition>, RwLockError> {
        if let Some(value_data) = self.inner.read()?.get(identifier) {
            if let ValueData::TypeDefinition(definition) = value_data {
                return Ok(Some(definition.clone()));
            }
        }

        for built_in_definition in all_built_in_type_definitions() {
            if built_in_definition.name() == identifier.inner().as_ref() {
                return Ok(Some(built_in_definition.get(self).clone()?));
            }
        }

        Ok(None)
    }

    /// Set a value to a key.
    pub fn set_value(&self, key: Identifier, value: Value) -> Result<(), RwLockError> {
        self.inner.write()?.insert(
            key,
            ValueData::Value {
                inner: value,
                runtime_uses: Arc::new(RwLock::new(0)),
            },
        );

        Ok(())
    }

    /// Set a type hint.
    ///
    /// This allows the interpreter to check a value's type before the value
    /// actually exists by predicting what the abstract tree will produce.
    pub fn set_type(&self, key: Identifier, r#type: Type) -> Result<(), RwLockError> {
        self.inner
            .write()?
            .insert(key, ValueData::TypeHint { inner: r#type });

        Ok(())
    }

    /// Set a type definition.
    ///
    /// This allows defined types (i.e. structs and enums) to be instantiated
    /// later while using this context.
    pub fn set_definition(
        &self,
        key: Identifier,
        definition: TypeDefinition,
    ) -> Result<(), RwLockError> {
        self.inner
            .write()?
            .insert(key, ValueData::TypeDefinition(definition));

        Ok(())
    }

    /// Remove a key-value pair.
    pub fn unset(&self, key: &Identifier) -> Result<(), RwLockError> {
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
    TypeHint {
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
                ValueData::TypeHint { inner: left_inner },
                ValueData::TypeHint { inner: right_inner },
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
                ValueData::TypeHint { inner: inner_left },
                ValueData::TypeHint { inner: inner_right },
            ) => inner_left.cmp(inner_right),
            (ValueData::TypeDefinition(left), ValueData::TypeDefinition(right)) => left.cmp(right),
            (ValueData::TypeDefinition(_), _) => Greater,
            (ValueData::TypeHint { .. }, _) => Less,
        }
    }
}
