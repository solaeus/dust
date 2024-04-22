use std::cmp::Ordering;

use crate::{
    context::Context,
    error::{RuntimeError, ValidationError},
};

use super::{AbstractNode, Action, Statement, Type};

#[derive(Clone, Debug)]
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
                let action = statement.clone().run(_context, _clear_variables)?;

                match action {
                    Action::Return(_) => {}
                    Action::None => {}
                    Action::Break => return Ok(Action::Break),
                }
            }
        }
    }
}

impl Eq for Loop {}

impl PartialEq for Loop {
    fn eq(&self, other: &Self) -> bool {
        self.statements.eq(&other.statements)
    }
}

impl PartialOrd for Loop {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Loop {
    fn cmp(&self, other: &Self) -> Ordering {
        self.statements.cmp(&other.statements)
    }
}
