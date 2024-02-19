//! A garbage-collecting execution context that stores variables and type data
//! during the [Interpreter][crate::Interpreter]'s abstraction and execution
//! process.
//!
//! ## Setting values
//!
//! When data is stored in a context, it can be accessed by dust source code.
//! This allows you to insert values and type definitions before any code is
//! interpreted.
//!
//! ```
//! # use dust_lang::*;
//! let context = Context::default();
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
//!
//! ## Garbage Collection
//!
//! To disable garbage collection, run a Context in AllowGarbage mode.
//!
//! ```
//! # use dust_lang::*;
//! let context = Context::new(ContextMode::AllowGarbage);   
//! ```
//!
//!
//! Every item stored in a Context has a counter attached to it. You must use
//! [Context::add_allowance][] to let the Context know not to drop the value.
//! Every time you use [Context::get_value][] it checks the number of times it
//! has been used and compares it to the number of allowances. If the limit
//! has been reached, the value will be removed from the context and can no
//! longer be found.
mod usage_counter;
mod value_data;

pub use usage_counter::UsageCounter;
pub use value_data::ValueData;

use std::{
    cmp::Ordering,
    collections::BTreeMap,
    fmt::Display,
    sync::{Arc, RwLock, RwLockReadGuard},
};

use crate::{
    built_in_type_definitions::all_built_in_type_definitions, built_in_values::all_built_in_values,
    error::rw_lock_error::RwLockError, Identifier, Type, TypeDefinition, Value,
};

#[derive(Clone, Debug, PartialEq)]
pub enum ContextMode {
    AllowGarbage,
    RemoveGarbage,
}

/// An execution context stores that variable and type data during the
/// [Interpreter]'s abstraction and execution process.
///
/// See the [module-level docs][self] for more info.
#[derive(Clone, Debug)]
pub struct Context {
    mode: ContextMode,
    inner: Arc<RwLock<BTreeMap<Identifier, (ValueData, UsageCounter)>>>,
}

impl Context {
    /// Return a new, empty Context.
    pub fn new(mode: ContextMode) -> Self {
        Self {
            mode,
            inner: Arc::new(RwLock::new(BTreeMap::new())),
        }
    }

    /// Return a lock guard to the inner BTreeMap.
    pub fn inner(
        &self,
    ) -> Result<RwLockReadGuard<BTreeMap<Identifier, (ValueData, UsageCounter)>>, RwLockError> {
        Ok(self.inner.read()?)
    }

    /// Create a new context with all of the data from an existing context.
    pub fn with_variables_from(other: &Context) -> Result<Context, RwLockError> {
        let mut new_variables = BTreeMap::new();

        for (identifier, (value_data, counter)) in other.inner.read()?.iter() {
            let (allowances, _runtime_uses) = counter.get_counts()?;
            let new_counter = UsageCounter::with_counts(allowances, 0);

            new_variables.insert(identifier.clone(), (value_data.clone(), new_counter));
        }

        Ok(Context {
            mode: other.mode.clone(),
            inner: Arc::new(RwLock::new(new_variables)),
        })
    }

    /// Modify a context to take the functions and type definitions of another.
    ///
    /// In the case of the conflict, the inherited value will override the previous
    /// value.
    pub fn inherit_from(&self, other: &Context) -> Result<(), RwLockError> {
        let mut self_variables = self.inner.write()?;

        for (identifier, (value_data, counter)) in other.inner.read()?.iter() {
            let (allowances, _runtime_uses) = counter.get_counts()?;
            let new_counter = UsageCounter::with_counts(allowances, 0);

            if let ValueData::Value(value) = value_data {
                if value.is_function() {
                    self_variables.insert(identifier.clone(), (value_data.clone(), new_counter));
                }
            } else if let ValueData::TypeHint(r#type) = value_data {
                if r#type.is_function() {
                    self_variables.insert(identifier.clone(), (value_data.clone(), new_counter));
                }
            } else if let ValueData::TypeDefinition(_) = value_data {
                self_variables.insert(identifier.clone(), (value_data.clone(), new_counter));
            }
        }

        Ok(())
    }

    /// Modify a context to take all the information of another.
    ///
    /// In the case of the conflict, the inherited value will override the previous
    /// value.
    ///
    /// ```
    /// # use dust_lang::*;
    /// let first_context = Context::default();
    /// let second_context = Context::default();
    ///
    /// second_context.set_value(
    ///     "Foo".into(),
    ///     Value::String("Bar".to_string())
    /// );
    ///
    /// first_context.inherit_all_from(&second_context).unwrap();
    ///
    /// assert_eq!(first_context, second_context);
    /// ```
    pub fn inherit_all_from(&self, other: &Context) -> Result<(), RwLockError> {
        let mut self_variables = self.inner.write()?;

        for (identifier, (value_data, _counter)) in other.inner.read()?.iter() {
            self_variables.insert(
                identifier.clone(),
                (value_data.clone(), UsageCounter::new()),
            );
        }

        Ok(())
    }

    /// Increment the number of allowances a variable has. Return a boolean
    /// representing whether or not the variable was found.
    pub fn add_allowance(&self, identifier: &Identifier) -> Result<bool, RwLockError> {
        if let Some((_value_data, counter)) = self.inner.read()?.get(identifier) {
            log::debug!("Adding allowance for {identifier}.");

            counter.add_allowance()?;

            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// Get a [Value] from the context.
    pub fn get_value(&self, identifier: &Identifier) -> Result<Option<Value>, RwLockError> {
        let (value, counter) =
            if let Some((value_data, counter)) = self.inner.read()?.get(identifier) {
                if let ValueData::Value(value) = value_data {
                    (value.clone(), counter.clone())
                } else {
                    return Ok(None);
                }
            } else {
                for built_in_value in all_built_in_values() {
                    if built_in_value.name() == identifier.inner().as_ref() {
                        return Ok(Some(built_in_value.get().clone()));
                    }
                }

                return Ok(None);
            };

        counter.add_runtime_use()?;

        log::debug!("Adding runtime use for {identifier}.");

        let (allowances, runtime_uses) = counter.get_counts()?;

        if self.mode == ContextMode::RemoveGarbage && allowances == runtime_uses {
            self.unset(identifier)?;
        }

        Ok(Some(value))
    }

    /// Get a [Type] from the context.
    ///
    /// If the key matches a stored [Value], its type will be returned. It if
    /// matches a type hint, the type hint will be returned.
    pub fn get_type(&self, identifier: &Identifier) -> Result<Option<Type>, RwLockError> {
        if let Some((value_data, _counter)) = self.inner.read()?.get(identifier) {
            match value_data {
                ValueData::Value(value) => return Ok(Some(value.r#type()?)),
                ValueData::TypeHint(r#type) => return Ok(Some(r#type.clone())),
                ValueData::TypeDefinition(_) => todo!(),
            }
        }

        for built_in_value in all_built_in_values() {
            if built_in_value.name() == identifier.inner().as_ref() {
                return Ok(Some(built_in_value.get().r#type()?));
            }
        }

        Ok(None)
    }

    /// Get a [TypeDefinition] from the context.
    ///
    /// This will also return a built-in type definition if one matches the key.
    /// See the [module-level docs][self] for more info.
    pub fn get_definition(
        &self,
        identifier: &Identifier,
    ) -> Result<Option<TypeDefinition>, RwLockError> {
        if let Some((value_data, _counter)) = self.inner.read()?.get(identifier) {
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
        let mut map = self.inner.write()?;
        let old_data = map.remove(&key);

        if let Some((_, old_counter)) = old_data {
            map.insert(key, (ValueData::Value(value), old_counter));
        } else {
            map.insert(key, (ValueData::Value(value), UsageCounter::new()));
        }

        Ok(())
    }

    /// Set a type hint.
    ///
    /// This allows the interpreter to check a value's type before the value
    /// actually exists by predicting what the abstract tree will produce.
    pub fn set_type(&self, key: Identifier, r#type: Type) -> Result<(), RwLockError> {
        self.inner
            .write()?
            .insert(key, (ValueData::TypeHint(r#type), UsageCounter::new()));

        Ok(())
    }

    /// Set a type definition.
    ///
    /// This allows defined types (i.e. structs and enums) to be instantiated
    /// later while running the interpreter using this context.
    pub fn set_definition(
        &self,
        key: Identifier,
        definition: TypeDefinition,
    ) -> Result<(), RwLockError> {
        self.inner.write()?.insert(
            key,
            (ValueData::TypeDefinition(definition), UsageCounter::new()),
        );

        Ok(())
    }

    /// Remove a key-value pair.
    pub fn unset(&self, key: &Identifier) -> Result<(), RwLockError> {
        log::debug!("Dropping variable {key}.");

        self.inner.write()?.remove(key);

        Ok(())
    }
}

impl Default for Context {
    fn default() -> Self {
        Context::new(ContextMode::RemoveGarbage)
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
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
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

impl Display for Context {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{{")?;

        for (identifier, value_data) in self.inner.read().unwrap().iter() {
            writeln!(f, "{identifier} {value_data:?}")?;
        }

        writeln!(f, "}}")
    }
}

#[cfg(test)]
mod tests {
    use crate::*;

    #[test]
    fn drops_variables() {
        let context = Context::default();

        interpret_with_context(
            "
                x = 1
                y = 2

                z = x + y
            ",
            context.clone(),
        )
        .unwrap();

        assert_eq!(context.inner.read().unwrap().len(), 1);
    }
}
