use crate::{
    context::Context,
    error::{RuntimeError, ValidationError},
};

use super::{AbstractNode, Action, Statement, Type};

#[derive(Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
pub struct Loop {
    statements: Vec<Statement>,
}

impl Loop {
    pub fn new(statements: Vec<Statement>) -> Self {
        Self { statements }
    }
}

impl AbstractNode for Loop {
    fn expected_type(&self, _context: &Context) -> Result<Type, ValidationError> {
        Ok(Type::None)
    }

    fn validate(&self, _context: &Context) -> Result<(), ValidationError> {
        for statement in &self.statements {
            statement.validate(_context)?;
        }

        Ok(())
    }

    fn run(self, _context: &mut Context, _clear_variables: bool) -> Result<Action, RuntimeError> {
        loop {
            for statement in &self.statements {
                let action = statement.clone().run(_context, false)?;

                match action {
                    Action::Return(_) => {}
                    Action::None => {}
                    Action::Break => return Ok(Action::Break),
                }
            }
        }
    }
}
