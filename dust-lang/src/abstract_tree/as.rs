use std::{
    borrow::Borrow,
    fmt::{self, Display, Formatter},
};

use serde::{Deserialize, Serialize};

use crate::{
    context::Context,
    error::{RuntimeError, ValidationError},
    value::ValueInner,
    Value,
};

use super::{AbstractNode, Evaluation, Expression, SourcePosition, Type, TypeConstructor};

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
    fn define_and_validate(
        &self,
        _context: &Context,
        _manage_memory: bool,
        scope: SourcePosition,
    ) -> Result<(), ValidationError> {
        self.expression
            .define_and_validate(_context, _manage_memory, scope)?;

        match self.constructor {
            TypeConstructor::Raw(_) => {}
            _ => todo!("Create an error for this occurence"),
        };

        Ok(())
    }

    fn evaluate(
        self,
        context: &Context,
        _manage_memory: bool,
        scope: SourcePosition,
    ) -> Result<Option<Evaluation>, RuntimeError> {
        let expression_position = self.expression.position();
        let evaluation = self.expression.evaluate(context, _manage_memory, scope)?;
        let value = if let Some(Evaluation::Return(value)) = evaluation {
            value
        } else {
            return Err(RuntimeError::ValidationFailure(
                ValidationError::ExpectedValueStatement(expression_position),
            ));
        };
        let r#type = self.constructor.construct(context)?;
        let (from_value, to_type): (&ValueInner, Type) = (value.inner().borrow(), r#type);

        let converted = match (from_value, to_type) {
            (_, Type::String) => Value::string(value.to_string()),
            _ => todo!("Create an error for this occurence"),
        };

        Ok(Some(Evaluation::Return(converted)))
    }

    fn expected_type(&self, context: &Context) -> Result<Option<Type>, ValidationError> {
        self.constructor.construct(context).map(Some)
    }
}

impl Display for As {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{} as {}", self.expression, self.constructor)
    }
}
