use std::{
    fmt::{self, Display, Formatter},
    sync::Mutex,
};

use rayon::prelude::*;
use serde::{Deserialize, Serialize};

use crate::{
    context::Context,
    error::{RuntimeError, ValidationError},
};

use super::{AbstractNode, Evaluation, Statement, Type};

#[derive(Clone, Debug, Eq, PartialEq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct AsyncBlock {
    statements: Vec<Statement>,
}

impl AsyncBlock {
    pub fn new(statements: Vec<Statement>) -> Self {
        Self { statements }
    }
}

impl AbstractNode for AsyncBlock {
    fn define_and_validate(
        &self,
        _context: &Context,
        manage_memory: bool,
    ) -> Result<(), ValidationError> {
        for statement in &self.statements {
            statement.define_and_validate(_context, manage_memory)?;
        }

        Ok(())
    }

    fn evaluate(self, _context: &Context, _: bool) -> Result<Option<Evaluation>, RuntimeError> {
        let final_result = Mutex::new(Ok(None));
        let statement_count = self.statements.len();
        let error_option = self.statements.into_par_iter().enumerate().find_map_any(
            |(index, statement)| -> Option<RuntimeError> {
                let result = statement.evaluate(&_context, false);

                if let Err(error) = result {
                    return Some(error);
                }

                if index == statement_count - 1 {
                    // It is safe to unwrap here because only one thread uses the Mutex
                    *final_result.lock().unwrap() = result;
                }

                None
            },
        );

        if let Some(error) = error_option {
            Err(error)
        } else {
            final_result.into_inner()?
        }
    }

    fn expected_type(&self, _context: &Context) -> Result<Option<Type>, ValidationError> {
        self.statements.last().unwrap().expected_type(_context)
    }
}

impl Display for AsyncBlock {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "async {{")?;

        for statement in &self.statements {
            write!(f, "{statement}")?;
        }

        write!(f, "}}")
    }
}
