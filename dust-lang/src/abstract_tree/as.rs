use std::borrow::Borrow;

use serde::{Deserialize, Serialize};

use crate::{
    context::Context,
    error::{RuntimeError, ValidationError},
    value::ValueInner,
    Value,
};

use super::{AbstractNode, Evaluation, ExpectedType, Expression, Type, TypeConstructor};

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
    fn validate(
        &self,
        _context: &mut Context,
        _manage_memory: bool,
    ) -> Result<(), ValidationError> {
        match self.constructor {
            TypeConstructor::Type(_) => {}
            _ => todo!("Create an error for this occurence."),
        };

        match self.expression.expected_type(_context)? {
            Type::Boolean | Type::Float | Type::Integer | Type::String => {}
            _ => todo!("Create an error for this occurence."),
        };

        Ok(())
    }

    fn evaluate(
        self,
        context: &mut Context,
        _manage_memory: bool,
    ) -> Result<Evaluation, RuntimeError> {
        let expression_position = self.expression.position();
        let action = self.expression.evaluate(context, _manage_memory)?;
        let value = if let Evaluation::Return(value) = action {
            value
        } else {
            return Err(RuntimeError::ValidationFailure(
                ValidationError::InterpreterExpectedReturn(expression_position),
            ));
        };
        let r#type = self.constructor.construct(&context)?;
        let (from_value, to_type): (&ValueInner, Type) = (value.inner().borrow(), r#type);

        let converted = match (from_value, to_type) {
            (ValueInner::Boolean(boolean), Type::String) => Value::string(boolean.to_string()),
            (ValueInner::Integer(integer), Type::String) => Value::string(integer.to_string()),
            _ => todo!("Create an error for this occurence."),
        };

        Ok(Evaluation::Return(converted))
    }
}

impl ExpectedType for As {
    fn expected_type(&self, context: &mut Context) -> Result<Type, ValidationError> {
        self.constructor.clone().construct(&context)
    }
}
