use std::{
    fmt::{self, Display, Formatter},
    sync::Arc,
};

use crate::{
    context::Context,
    error::{RuntimeError, ValidationError},
    Value,
};

use super::{AbstractTree, Type};

#[derive(Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
pub struct Identifier(Arc<String>);

impl Identifier {
    pub fn new<T: ToString>(string: T) -> Self {
        Identifier(Arc::new(string.to_string()))
    }

    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

impl AbstractTree for Identifier {
    fn expected_type(&self, context: &Context) -> Result<Type, ValidationError> {
        if let Some(r#type) = context.get_type(self)? {
            Ok(r#type)
        } else {
            Err(ValidationError::VariableNotFound(self.clone()))
        }
    }

    fn validate(&self, context: &Context) -> Result<(), ValidationError> {
        if let Some(_) = context.get_data(self)? {
            Ok(())
        } else {
            Err(ValidationError::VariableNotFound(self.clone()))
        }
    }

    fn run(self, context: &Context) -> Result<Value, RuntimeError> {
        let value = context
            .get_value(&self)?
            .unwrap_or_else(Value::none)
            .clone();

        Ok(value)
    }
}

impl Display for Identifier {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}
