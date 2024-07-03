use std::{
    collections::HashMap,
    fmt::Debug,
    sync::{Arc, RwLock},
};

use crate::{
    abstract_tree::Type,
    error::{PoisonError, ValidationError},
    identifier::Identifier,
    standard_library::core_context,
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

    pub fn new_with_std_core(parent: Option<Context>) -> Result<Self, PoisonError> {
        let new = Context::with_variables_from(core_context())?;

        if let Some(context) = parent {
            new.set_parent(context)?;
        }

        Ok(new)
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

        let data = self.data.read()?;

        if let Some(_) = data.variables.get(identifier) {
            Ok(true)
        } else if let Some(parent) = &data.parent {
            parent.contains_inheritable(identifier)
        } else {
            Ok(false)
        }
    }

    fn contains_inheritable(&self, identifier: &Identifier) -> Result<bool, ValidationError> {
        let data = self.data.read()?;

        if let Some((variable_data, _)) = data.variables.get(identifier) {
            match variable_data {
                VariableData::Type(Type::Enum { .. })
                | VariableData::Type(Type::Function { .. })
                | VariableData::Type(Type::Structure { .. }) => return Ok(true),
                VariableData::Value(value) => match value.inner().as_ref() {
                    ValueInner::BuiltInFunction(_) | ValueInner::Function(_) => return Ok(true),
                    _ => {}
                },
                _ => {}
            }
        }

        if let Some(parent) = &data.parent {
            parent.contains_inheritable(identifier)
        } else {
            Ok(false)
        }
    }

    pub fn get_type(&self, identifier: &Identifier) -> Result<Option<Type>, ValidationError> {
        log::trace!("Getting {identifier}'s type");

        let data = self.data.read()?;

        if let Some((data, _)) = data.variables.get(identifier) {
            let r#type = match data {
                VariableData::Type(r#type) => r#type.clone(),
                VariableData::Value(value) => value.r#type(self)?,
            };

            Ok(Some(r#type.clone()))
        } else if let Some(parent) = &data.parent {
            parent.get_inheritable_type(identifier)
        } else {
            Ok(None)
        }
    }

    fn get_inheritable_type(
        &self,
        identifier: &Identifier,
    ) -> Result<Option<Type>, ValidationError> {
        let data = self.data.read()?;

        if let Some(r#type) = self.get_type(identifier)? {
            match r#type {
                Type::Enum { .. } | Type::Function { .. } | Type::Structure { .. } => {
                    return Ok(Some(r#type))
                }
                _ => {}
            }
        }

        if let Some(parent) = &data.parent {
            parent.get_inheritable_type(identifier)
        } else {
            Ok(None)
        }
    }

    pub fn use_value(&self, identifier: &Identifier) -> Result<Option<Value>, PoisonError> {
        log::trace!("Using {identifier}'s value");

        let data = self.data.read()?;

        if let Some((VariableData::Value(value), usage_data)) = data.variables.get(identifier) {
            usage_data.inner().write()?.actual += 1;
            *self.is_clean.write()? = false;

            return Ok(Some(value.clone()));
        } else if let Some(parent) = &data.parent {
            parent.use_inheritable_value(identifier)
        } else {
            Ok(None)
        }
    }

    fn use_inheritable_value(&self, identifier: &Identifier) -> Result<Option<Value>, PoisonError> {
        let data = self.data.read()?;

        if let Some((VariableData::Value(value), usage_data)) = data.variables.get(identifier) {
            match value.inner().as_ref() {
                ValueInner::BuiltInFunction(_) | ValueInner::Function(_) => {
                    usage_data.inner().write()?.actual += 1;
                    *self.is_clean.write()? = false;

                    return Ok(Some(value.clone()));
                }
                _ => {}
            }
        }

        if let Some(parent) = &data.parent {
            parent.use_inheritable_value(identifier)
        } else {
            Ok(None)
        }
    }

    pub fn get_value(&self, identifier: &Identifier) -> Result<Option<Value>, PoisonError> {
        log::trace!("Getting {identifier}'s value");

        let data = self.data.read()?;

        if let Some((VariableData::Value(value), _)) = data.variables.get(identifier) {
            Ok(Some(value.clone()))
        } else if let Some(parent) = &data.parent {
            parent.get_inheritable_value(identifier)
        } else {
            Ok(None)
        }
    }

    fn get_inheritable_value(&self, identifier: &Identifier) -> Result<Option<Value>, PoisonError> {
        if let Some(value) = self.get_value(identifier)? {
            match value.inner().as_ref() {
                ValueInner::BuiltInFunction(_) | ValueInner::Function(_) => return Ok(Some(value)),
                _ => {}
            }
        }

        let data = self.data.read()?;

        if let Some(parent) = &data.parent {
            parent.get_inheritable_value(identifier)
        } else {
            Ok(None)
        }
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
        log::trace!("Adding expected use for variable {identifier}");

        let data = self.data.read()?;

        if let Some((_, usage_data)) = data.variables.get(identifier) {
            usage_data.inner().write()?.expected += 1;

            return Ok(true);
        } else if let Some(parent) = &data.parent {
            parent.add_expected_use_for_inheritable(identifier)
        } else {
            Ok(false)
        }
    }

    fn add_expected_use_for_inheritable(
        &self,
        identifier: &Identifier,
    ) -> Result<bool, PoisonError> {
        let data = self.data.read()?;

        if let Some((variable_data, usage_data)) = data.variables.get(identifier) {
            match variable_data {
                VariableData::Type(Type::Enum { .. })
                | VariableData::Type(Type::Function { .. })
                | VariableData::Type(Type::Structure { .. }) => {
                    usage_data.inner().write()?.expected += 1;

                    return Ok(true);
                }
                VariableData::Value(value) => match value.inner().as_ref() {
                    ValueInner::BuiltInFunction(_) | ValueInner::Function(_) => {
                        usage_data.inner().write()?.expected += 1;

                        return Ok(true);
                    }
                    _ => {}
                },
                _ => {}
            }
        }

        if let Some(parent) = &data.parent {
            parent.add_expected_use_for_inheritable(identifier)
        } else {
            Ok(false)
        }
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
