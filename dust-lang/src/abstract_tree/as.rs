use std::borrow::Borrow;

use serde::{Deserialize, Serialize};

use crate::{
    context::Context,
    error::{RuntimeError, ValidationError},
    value::ValueInner,
    Value,
};

use super::{AbstractNode, Action, ExpectedType, Type, ValueExpression, WithPosition};

#[derive(Clone, Debug, Eq, PartialEq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct As {
    expression: ValueExpression,
    r#type: WithPosition<Type>,
}

impl As {
    pub fn new(expression: ValueExpression, r#type: WithPosition<Type>) -> Self {
        Self { expression, r#type }
    }
}

impl AbstractNode for As {
    fn validate(
        &self,
        _context: &mut Context,
        _manage_memory: bool,
    ) -> Result<(), ValidationError> {
        match self.r#type.node {
            Type::Boolean | Type::Float | Type::Integer | Type::String => {}
            _ => todo!("Create an error for this occurence."),
        };

        match self.expression.expected_type(_context)? {
            Type::Boolean | Type::Float | Type::Integer | Type::String => Ok(()),
            _ => todo!("Create an error for this occurence."),
        }
    }

    fn run(self, _context: &mut Context, _manage_memory: bool) -> Result<Action, RuntimeError> {
        let expression_position = self.expression.position();
        let action = self.expression.run(_context, _manage_memory)?;
        let value = if let Action::Return(value) = action {
            value
        } else {
            return Err(RuntimeError::ValidationFailure(
                ValidationError::InterpreterExpectedReturn(expression_position),
            ));
        };
        let (from_value, to_type): (&ValueInner, Type) = (value.inner().borrow(), self.r#type.node);

        let converted = match (from_value, to_type) {
            (ValueInner::Boolean(boolean), Type::String) => Value::string(boolean.to_string()),
            (ValueInner::Integer(integer), Type::String) => Value::string(integer.to_string()),
            _ => todo!("Create an error for this occurence."),
        };

        Ok(Action::Return(converted))
    }
}

impl ExpectedType for As {
    fn expected_type(&self, _: &mut Context) -> Result<Type, ValidationError> {
        Ok(self.r#type.node.clone())
    }
}
