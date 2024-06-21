use serde::{Deserialize, Serialize};

use crate::{
    context::Context,
    error::{RuntimeError, ValidationError},
};

use super::{Evaluation, Run, Statement, Validate};

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

impl Validate for Loop {
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
}

impl Run for Loop {
    fn run(
        self,
        _context: &mut Context,
        _manage_memory: bool,
    ) -> Result<Option<Evaluation>, RuntimeError> {
        loop {
            for statement in &self.statements {
                let run = statement.clone().run(_context, false)?;

                if let Some(Evaluation::Break) = run {
                    return Ok(run);
                }
            }
        }
    }
}
