use std::{
    fmt::{self, Display, Formatter},
    sync::Arc,
};

use crate::{
    context::Context,
    error::{RuntimeError, ValidationError},
};

use super::{AbstractTree, Action, Type};

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
            Err(ValidationError::VariableNotFound {
                identifier: todo!(),
                position: todo!(),
            })
        }
    }

    fn validate(&self, context: &Context) -> Result<(), ValidationError> {
        if context.contains(self)? {
            Ok(())
        } else {
            Err(ValidationError::VariableNotFound {
                identifier: todo!(),
                position: todo!(),
            })
        }
    }

    fn run(self, context: &Context) -> Result<Action, RuntimeError> {
        let return_action = context.get_value(&self)?.map(|value| Action::Return(value));

        if let Some(action) = return_action {
            Ok(action)
        } else {
            Err(RuntimeError::ValidationFailure(
                ValidationError::VariableNotFound {
                    identifier: todo!(),
                    position: todo!(),
                },
            ))
        }
    }
}

impl Display for Identifier {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}
