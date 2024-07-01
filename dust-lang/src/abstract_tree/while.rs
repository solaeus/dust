use std::fmt::{self, Display, Formatter};

use serde::{Deserialize, Serialize};

use crate::{
    context::Context,
    error::{RuntimeError, ValidationError},
    value::ValueInner,
    Value,
};

use super::{AbstractNode, Evaluation, Expression, Statement, Type};

#[derive(Clone, Debug, Eq, PartialEq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct While {
    expression: Expression,
    statements: Vec<Statement>,
}

impl While {
    pub fn new(expression: Expression, statements: Vec<Statement>) -> Self {
        Self {
            expression,
            statements,
        }
    }
}

impl AbstractNode for While {
    fn define_types(&self, _context: &Context) -> Result<(), ValidationError> {
        self.expression.define_types(_context)?;

        for statement in &self.statements {
            statement.define_types(_context)?;
        }

        Ok(())
    }

    fn validate(&self, _context: &Context, _manage_memory: bool) -> Result<(), ValidationError> {
        self.expression.validate(_context, false)?;

        for statement in &self.statements {
            statement.validate(_context, false)?;
        }

        Ok(())
    }

    fn evaluate(
        self,
        _context: &Context,
        _manage_memory: bool,
    ) -> Result<Option<Evaluation>, RuntimeError> {
        let get_boolean = || -> Result<Value, RuntimeError> {
            let expression_position = self.expression.position();
            let evaluation = self
                .expression
                .clone()
                .evaluate(&mut _context.clone(), false)?;

            if let Some(Evaluation::Return(value)) = evaluation {
                Ok(value)
            } else {
                Err(RuntimeError::ValidationFailure(
                    ValidationError::ExpectedValueStatement(expression_position),
                ))
            }
        };

        while let ValueInner::Boolean(true) = get_boolean()?.inner().as_ref() {
            for statement in &self.statements {
                let evaluation = statement.clone().evaluate(&mut _context.clone(), false)?;

                if let Some(Evaluation::Break) = evaluation {
                    return Ok(evaluation);
                }
            }
        }

        Ok(None)
    }

    fn expected_type(&self, _context: &Context) -> Result<Option<Type>, ValidationError> {
        self.statements.last().unwrap().expected_type(_context)
    }
}

impl Display for While {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "while {} {{", self.expression)?;

        for statement in &self.statements {
            write!(f, "{statement}")?;
        }

        write!(f, "}}")
    }
}
