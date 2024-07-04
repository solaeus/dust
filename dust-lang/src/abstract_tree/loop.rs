use std::fmt::{self, Display, Formatter};

use serde::{Deserialize, Serialize};

use crate::{
    context::Context,
    error::{RuntimeError, ValidationError},
};

use super::{AbstractNode, Evaluation, SourcePosition, Statement, Type};

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
    fn define_and_validate(
        &self,
        _context: &Context,
        _manage_memory: bool,
        scope: SourcePosition,
    ) -> Result<(), ValidationError> {
        for statement in &self.statements {
            statement.define_and_validate(_context, false, scope)?;
        }

        Ok(())
    }

    fn evaluate(
        self,
        _context: &Context,
        _manage_memory: bool,
        scope: SourcePosition,
    ) -> Result<Option<Evaluation>, RuntimeError> {
        loop {
            for statement in &self.statements {
                let run = statement.clone().evaluate(_context, false, scope)?;

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

impl Display for Loop {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "loop {{ ")?;

        for statement in &self.statements {
            write!(f, "{statement}")?;
        }

        write!(f, " }}")
    }
}
