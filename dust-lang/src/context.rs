use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};

use crate::{
    abstract_tree::Type,
    error::{PoisonError, ValidationError},
    identifier::Identifier,
    value::ValueInner,
    Value,
};

#[derive(Clone, Debug)]
pub struct Context {
    data: Arc<RwLock<ContextData>>,
    is_clean: Arc<RwLock<bool>>,
}

#[derive(Clone, Debug)]
struct ContextData {
    variables: HashMap<Identifier, (VariableData, UsageData)>,
    parent: Option<Context>,
}

impl Context {
    pub fn new(parent: Option<Context>) -> Self {
        Context {
            data: Arc::new(RwLock::new(ContextData {
                variables: HashMap::new(),
                parent,
            })),
            is_clean: Arc::new(RwLock::new(true)),
        }
    }

    pub fn with_variables_from(other: &Context) -> Result<Self, PoisonError> {
        let variables = other.data.read()?.variables.clone();

        Ok(Context {
            data: Arc::new(RwLock::new(ContextData {
                variables,
                parent: None,
            })),
            is_clean: Arc::new(RwLock::new(true)),
        })
    }

    pub fn create_child(&self) -> Context {
        Context::new(Some(self.clone()))
    }

    pub fn set_parent(&self, parent: Context) -> Result<(), PoisonError> {
        self.data.write()?.parent = Some(parent);

        Ok(())
    }

    pub fn contains(&self, identifier: &Identifier) -> Result<bool, ValidationError> {
        log::trace!("Checking that {identifier} exists");

        Ok(self.get_type(identifier)?.is_some())
    }

    pub fn get_type(&self, identifier: &Identifier) -> Result<Option<Type>, ValidationError> {
        log::trace!("Getting {identifier}'s type");

        let data = self.data.read()?;

        if let Some((data, _)) = data.variables.get(identifier) {
            let r#type = match data {
                VariableData::Type(r#type) => r#type.clone(),
                VariableData::Value(value) => value.r#type(self)?,
            };

            return Ok(Some(r#type.clone()));
        } else if let Some(parent) = &data.parent {
            if let Some(r#type) = parent.get_type(identifier)? {
                match r#type {
                    Type::Enum { .. } | Type::Function { .. } | Type::Structure { .. } => {
                        return Ok(Some(r#type))
                    }
                    _ => {}
                }
            }
        }

        Ok(None)
    }

    pub fn use_value(&self, identifier: &Identifier) -> Result<Option<Value>, PoisonError> {
        log::trace!("Using {identifier}'s value");

        let data = self.data.read()?;

        if let Some((VariableData::Value(value), usage_data)) = data.variables.get(identifier) {
            usage_data.inner().write()?.actual += 1;
            *self.is_clean.write()? = false;

            return Ok(Some(value.clone()));
        } else if let Some(parent) = &data.parent {
            if let Some(value) = parent.get_value(identifier)? {
                match value.inner().as_ref() {
                    ValueInner::EnumInstance { .. }
                    | ValueInner::Function(_)
                    | ValueInner::Structure { .. }
                    | ValueInner::BuiltInFunction(_) => return Ok(Some(value)),
                    _ => {}
                }
            }
        }

        Ok(None)
    }

    pub fn get_value(&self, identifier: &Identifier) -> Result<Option<Value>, PoisonError> {
        log::trace!("Getting {identifier}'s value");

        let data = self.data.read()?;

        if let Some((VariableData::Value(value), _)) = data.variables.get(identifier) {
            return Ok(Some(value.clone()));
        } else if let Some(parent) = &data.parent {
            if let Some(value) = parent.get_value(identifier)? {
                match value.inner().as_ref() {
                    ValueInner::EnumInstance { .. }
                    | ValueInner::Function(_)
                    | ValueInner::Structure { .. }
                    | ValueInner::BuiltInFunction(_) => return Ok(Some(value)),
                    _ => {}
                }
            }
        }

        Ok(None)
    }

    pub fn set_type(&self, identifier: Identifier, r#type: Type) -> Result<(), PoisonError> {
        log::debug!("Setting {identifier} to type {}", r#type);

        self.data
            .write()?
            .variables
            .insert(identifier, (VariableData::Type(r#type), UsageData::new()));

        Ok(())
    }

    pub fn set_value(&self, identifier: Identifier, value: Value) -> Result<(), PoisonError> {
        log::debug!("Setting {identifier} to value {value}");

        let mut data = self.data.write()?;
        let usage_data = data
            .variables
            .remove(&identifier)
            .map(|(_, usage_data)| usage_data)
            .unwrap_or_else(|| UsageData::new());

        data.variables
            .insert(identifier, (VariableData::Value(value), usage_data));

        Ok(())
    }

    pub fn add_expected_use(&self, identifier: &Identifier) -> Result<bool, PoisonError> {
        let data = self.data.read()?;

        if let Some((_, usage_data)) = data.variables.get(identifier) {
            log::trace!("Adding expected use for variable {identifier}");

            usage_data.inner().write()?.expected += 1;

            return Ok(true);
        } else if let Some(parent) = &data.parent {
            let parent_data = parent.data.read()?;

            if let Some((variable_data, usage_data)) = parent_data.variables.get(identifier) {
                if let VariableData::Value(value) = variable_data {
                    if let ValueInner::Function(_) = value.inner().as_ref() {
                        usage_data.inner().write()?.expected += 1;

                        return Ok(true);
                    }
                }
            }
        }

        Ok(false)
    }

    pub fn clean(&self) -> Result<(), PoisonError> {
        if *self.is_clean.read()? {
            return Ok(());
        }

        self.data.write()?.variables.retain(
            |identifier, (value_data, usage_data)| match value_data {
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
        Context::new(None)
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
