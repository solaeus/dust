use std::borrow::Borrow;

use serde::{Deserialize, Serialize};

use crate::{
    context::Context,
    error::{RuntimeError, ValidationError},
    value::ValueInner,
    Value,
};

use super::{AbstractNode, Evaluation, Expression, Type, TypeConstructor};

#[derive(Clone, Debug, Eq, PartialEq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct As {
    expression: Expression,
    constructor: TypeConstructor,
}

impl As {
    pub fn new(expression: Expression, constructor: TypeConstructor) -> Self {
        Self {
            expression,
            constructor,
        }
    }
}

impl AbstractNode for As {
    fn define_types(&self, _: &Context) -> Result<(), ValidationError> {
        Ok(())
    }

    fn validate(&self, _context: &Context, _manage_memory: bool) -> Result<(), ValidationError> {
        match self.constructor {
            TypeConstructor::Raw(_) => {}
            _ => todo!("Create an error for this occurence."),
        };

        match self.expression.expected_type(_context)? {
            Some(Type::Boolean) | Some(Type::Float) | Some(Type::Integer) | Some(Type::String) => {}
            _ => todo!("Create an error for this occurence."),
        };

        Ok(())
    }

    fn evaluate(
        self,
        context: &Context,
        _manage_memory: bool,
    ) -> Result<Option<Evaluation>, RuntimeError> {
        let expression_position = self.expression.position();
        let evaluation = self.expression.evaluate(context, _manage_memory)?;
        let value = if let Some(Evaluation::Return(value)) = evaluation {
            value
        } else {
            return Err(RuntimeError::ValidationFailure(
                ValidationError::ExpectedExpression(expression_position),
            ));
        };
        let r#type = self.constructor.construct(&context)?;
        let (from_value, to_type): (&ValueInner, Type) = (value.inner().borrow(), r#type);

        let converted = match (from_value, to_type) {
            (ValueInner::Boolean(boolean), Type::String) => Value::string(boolean.to_string()),
            (ValueInner::Integer(integer), Type::String) => Value::string(integer.to_string()),
            _ => todo!("Create an error for this occurence."),
        };

        Ok(Some(Evaluation::Return(converted)))
    }

    fn expected_type(&self, context: &Context) -> Result<Option<Type>, ValidationError> {
        self.constructor
            .construct(&context)
            .map(|r#type| Some(r#type))
    }
}
