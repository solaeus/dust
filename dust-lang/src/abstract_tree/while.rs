use serde::{Deserialize, Serialize};

use crate::{
    context::Context,
    error::{RuntimeError, ValidationError},
    value::ValueInner,
    Value,
};

use super::{AbstractNode, Evaluation, Expression, Statement, Type, Validate};

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

impl Validate for While {
    fn validate(
        &self,
        _context: &mut Context,
        _manage_memory: bool,
    ) -> Result<(), ValidationError> {
        self.expression.validate(_context, false)?;

        for statement in &self.statements {
            statement.validate(_context, false)?;
        }

        Ok(())
    }
}

impl AbstractNode for While {
    fn evaluate(
        self,
        _context: &mut Context,
        _manage_memory: bool,
    ) -> Result<Option<Evaluation>, RuntimeError> {
        let get_boolean = || -> Result<Value, RuntimeError> {
            let expression_position = self.expression.position();
            let action = self
                .expression
                .clone()
                .evaluate(&mut _context.clone(), false)?;

            if let Evaluation::Return(value) = action {
                Ok(value)
            } else {
                Err(RuntimeError::ValidationFailure(
                    ValidationError::InterpreterExpectedReturn(expression_position),
                ))
            }
        };

        while let ValueInner::Boolean(true) = get_boolean()?.inner().as_ref() {
            for statement in &self.statements {
                let evaluation = statement.clone().run(&mut _context.clone(), false)?;

                if let Some(Evaluation::Break) = evaluation {
                    return Ok(evaluation);
                }
            }
        }

        Ok(None)
    }

    fn expected_type(&self, _context: &mut Context) -> Result<Option<Type>, ValidationError> {
        self.statements.last().unwrap().expected_type(_context)
    }
}
