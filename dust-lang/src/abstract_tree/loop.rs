use serde::{Deserialize, Serialize};

use crate::{
    context::Context,
    error::{RuntimeError, ValidationError},
};

use super::{AbstractNode, Evaluation, Statement, Type};

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
impl AbstractNode for Loop {
    fn define_types(&self, _context: &Context) -> Result<(), ValidationError> {
        for statement in &self.statements {
            statement.define_types(_context)?;
        }

        Ok(())
    }

    fn validate(&self, _context: &Context, _manage_memory: bool) -> Result<(), ValidationError> {
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
        loop {
            for statement in &self.statements {
                let run = statement.clone().run(_context, false)?;

                if let Some(Evaluation::Break) = run {
                    return Ok(run);
                }
            }
        }
    }

    fn expected_type(&self, _context: &Context) -> Result<Option<Type>, ValidationError> {
        self.last_statement().expected_type(_context)
    }
}
