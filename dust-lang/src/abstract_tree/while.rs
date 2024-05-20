use crate::{
    context::Context,
    error::{RuntimeError, ValidationError},
    value::ValueInner,
    Value,
};

use super::{AbstractNode, Action, Expression, Statement, Type};

#[derive(Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
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
    fn expected_type(&self, _context: &mut Context) -> Result<Type, ValidationError> {
        Ok(Type::None)
    }

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

    fn run(self, _context: &mut Context, _manage_memory: bool) -> Result<Action, RuntimeError> {
        let get_boolean = || -> Result<Value, RuntimeError> {
            let expression_position = self.expression.position();
            let action = self.expression.clone().run(&mut _context.clone(), false)?;

            if let Action::Return(value) = action {
                Ok(value)
            } else {
                Err(RuntimeError::ValidationFailure(
                    ValidationError::InterpreterExpectedReturn(expression_position),
                ))
            }
        };

        while let ValueInner::Boolean(true) = get_boolean()?.inner().as_ref() {
            for statement in &self.statements {
                let action = statement.clone().run(&mut _context.clone(), false)?;

                match action {
                    Action::Return(_) => {}
                    Action::None => {}
                    Action::Break => return Ok(Action::Break),
                }
            }
        }

        Ok(Action::None)
    }
}
