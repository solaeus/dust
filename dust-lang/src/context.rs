use std::{
    cmp::Ordering,
    collections::HashMap,
    fmt::Debug,
    sync::{Arc, RwLock},
};

use log::trace;
use rand::random;

use crate::{
    abstract_tree::{SourcePosition, Type},
    error::{PoisonError, ValidationError},
    identifier::Identifier,
    standard_library::core_context,
    Value,
};

#[derive(Clone, Debug)]
pub struct Context {
    id: u32,
    variables: Arc<RwLock<HashMap<Identifier, (VariableData, UsageData, SourcePosition)>>>,
    is_clean: Arc<RwLock<bool>>,
}

impl Context {
    pub fn new() -> Self {
        Context {
            id: random(),
            variables: Arc::new(RwLock::new(HashMap::new())),
            is_clean: Arc::new(RwLock::new(true)),
        }
    }

    pub fn new_with_std_core() -> Result<Self, PoisonError> {
        let new = Context::with_variables_from(core_context())?;

        Ok(new)
    }

    pub fn with_variables_from(other: &Context) -> Result<Self, PoisonError> {
        let variables = other.variables.read()?.clone();

        Ok(Context {
            id: random(),
            variables: Arc::new(RwLock::new(variables)),
            is_clean: Arc::new(RwLock::new(true)),
        })
    }

    pub fn inherit_variables_from(&self, other: &Context) -> Result<(), PoisonError> {
        let (get_self_variables, get_other_variables) =
            (self.variables.try_write(), other.variables.try_read());

        if let (Ok(mut self_variables), Ok(other_variables)) =
            (get_self_variables, get_other_variables)
        {
            self_variables.extend(other_variables.iter().map(|(identifier, data)| {
                trace!("Inheriting {identifier}");

                (identifier.clone(), data.clone())
            }));
        }

        Ok(())
    }

    pub fn contains(
        &self,
        identifier: &Identifier,
        scope: SourcePosition,
    ) -> Result<bool, ValidationError> {
        log::trace!("Checking that {identifier} exists");

        let variables = self.variables.read()?;

        if let Some((_, _, variable_scope)) = variables.get(identifier) {
            if scope.0 >= variable_scope.0 && scope.1 <= variable_scope.1 {
                return Ok(true);
            }
        } else {
            trace!("Denying access to {identifier}, out of scope")
        }

        Ok(false)
    }

    pub fn get_type(&self, identifier: &Identifier) -> Result<Option<Type>, ValidationError> {
        log::trace!("Getting {identifier}'s type");

        let variables = self.variables.read()?;

        if let Some((data, _, _)) = variables.get(identifier) {
            let r#type = match data {
                VariableData::Type(r#type) => r#type.clone(),
                VariableData::Value(value) => value.r#type(self)?,
            };

            Ok(Some(r#type.clone()))
        } else {
            Ok(None)
        }
    }

    pub fn use_value(&self, identifier: &Identifier) -> Result<Option<Value>, PoisonError> {
        log::trace!("Using {identifier}'s value");

        let variables = self.variables.read()?;

        if let Some((VariableData::Value(value), usage_data, _)) = variables.get(identifier) {
            usage_data.inner().write()?.actual += 1;
            *self.is_clean.write()? = false;

            return Ok(Some(value.clone()));
        } else {
            Ok(None)
        }
    }

    pub fn get_value(&self, identifier: &Identifier) -> Result<Option<Value>, PoisonError> {
        log::trace!("Getting {identifier}'s value");

        let variables = self.variables.read()?;

        if let Some((VariableData::Value(value), _, _)) = variables.get(identifier) {
            Ok(Some(value.clone()))
        } else {
            Ok(None)
        }
    }

    pub fn set_type(
        &self,
        identifier: Identifier,
        r#type: Type,
        scope: SourcePosition,
    ) -> Result<(), PoisonError> {
        log::debug!("Setting {identifier} to type {}", r#type);

        let mut variables = self.variables.write()?;
        let (usage_data, scope) = variables
            .remove(&identifier)
            .map(|(_, old_usage_data, old_scope)| (old_usage_data, old_scope))
            .unwrap_or_else(|| (UsageData::new(), scope));

        variables.insert(identifier, (VariableData::Type(r#type), usage_data, scope));

        Ok(())
    }

    pub fn set_value(
        &self,
        identifier: Identifier,
        value: Value,
        scope: SourcePosition,
    ) -> Result<(), PoisonError> {
        log::debug!("Setting {identifier} to value {value}");

        let mut variables = self.variables.write()?;
        let (usage_data, scope) = variables
            .remove(&identifier)
            .map(|(_, old_usage_data, old_scope)| (old_usage_data, old_scope))
            .unwrap_or_else(|| (UsageData::new(), scope));

        variables.insert(identifier, (VariableData::Value(value), usage_data, scope));

        Ok(())
    }

    pub fn add_expected_use(&self, identifier: &Identifier) -> Result<bool, PoisonError> {
        log::trace!("Adding expected use for variable {identifier}");

        let variables = self.variables.read()?;

        if let Some((_, usage_data, _)) = variables.get(identifier) {
            usage_data.inner().write()?.expected += 1;

            return Ok(true);
        } else {
            Ok(false)
        }
    }

    pub fn clean(&self) -> Result<(), PoisonError> {
        if *self.is_clean.read()? {
            return Ok(());
        }

        self.variables.write()?.retain(
            |identifier, (value_data, usage_data, _)| match value_data {
                VariableData::Type(_) => true,
                VariableData::Value(_) => {
                    let usage = usage_data.inner().read().unwrap();

                    if usage.actual < usage.expected {
                        true
                    } else {
                        log::trace!("Removing {identifier}");

                        false
                    }
                }
            },
        );

        *self.is_clean.write()? = true;

        Ok(())
    }

    pub fn is_clean(&mut self) -> Result<bool, PoisonError> {
        Ok(*self.is_clean.read()?)
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
        self.id == other.id
    }
}

impl PartialOrd for Context {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Context {
    fn cmp(&self, other: &Self) -> Ordering {
        self.id.cmp(&other.id)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum VariableData {
    Type(Type),
    Value(Value),
}

#[derive(Clone, Debug)]
pub struct UsageData(Arc<RwLock<UsageDataInner>>);

impl UsageData {
    pub fn inner(&self) -> &Arc<RwLock<UsageDataInner>> {
        &self.0
    }
}

#[derive(Clone, Debug)]
pub struct UsageDataInner {
    pub actual: u32,
    pub expected: u32,
}

impl UsageData {
    pub fn new() -> Self {
        UsageData(Arc::new(RwLock::new(UsageDataInner {
            actual: 0,
            expected: 0,
        })))
    }
}
