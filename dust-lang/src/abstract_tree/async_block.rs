use std::sync::Mutex;

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
    fn define_types(&self, _context: &Context) -> Result<(), ValidationError> {
        for statement in &self.statements {
            statement.define_types(_context)?;
        }

        Ok(())
    }

    fn validate(&self, _context: &Context, manage_memory: bool) -> Result<(), ValidationError> {
        for statement in &self.statements {
            statement.validate(_context, manage_memory)?;
        }

        Ok(())
    }

    fn evaluate(
        self,
        _context: &Context,
        _manage_memory: bool,
    ) -> Result<Option<Evaluation>, RuntimeError> {
        let statement_count = self.statements.len();
        let final_result = Mutex::new(Ok(None));

        self.statements
            .into_par_iter()
            .enumerate()
            .find_map_any(
                |(index, statement)| -> Option<Result<Option<Evaluation>, RuntimeError>> {
                    let result = statement.run(&mut _context.clone(), false);

                    if result.is_err() {
                        return Some(result);
                    }

                    if index == statement_count - 1 {
                        // It is safe to unwrap here because only one thread uses the Mutex
                        *final_result.lock().unwrap() = result;
                    }

                    None
                },
            )
            .unwrap_or(final_result.into_inner()?)
    }

    fn expected_type(&self, _context: &Context) -> Result<Option<Type>, ValidationError> {
        self.statements.last().unwrap().expected_type(_context)
    }
}
