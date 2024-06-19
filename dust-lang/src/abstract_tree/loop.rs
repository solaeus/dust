use serde::{Deserialize, Serialize};

use crate::{
    context::Context,
    error::{RuntimeError, ValidationError},
};

use super::{Evaluate, Evaluation, Statement};

#[derive(Clone, Debug, Eq, PartialEq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Loop {
    statements: Vec<Statement>,
}

impl Loop {
    pub fn new(statements: Vec<Statement>) -> Self {
        Self { statements }
    }

    pub fn last_statement(&self) -> &Statement {
        self.statements.last().unwrap()
    }
}

impl Evaluate for Loop {
    fn validate(
        &self,
        _context: &mut Context,
        _manage_memory: bool,
    ) -> Result<(), ValidationError> {
        for statement in &self.statements {
            statement.validate(_context, false)?;
        }

        Ok(())
    }

    fn evaluate(
        self,
        _context: &mut Context,
        _manage_memory: bool,
    ) -> Result<Evaluation, RuntimeError> {
        loop {
            for statement in &self.statements {
                let action = statement.clone().evaluate(_context, false)?;

                match action {
                    Evaluation::Return(_) => {}
                    Evaluation::None => {}
                    Evaluation::Break => return Ok(Evaluation::Break),
                }
            }
        }
    }
}
