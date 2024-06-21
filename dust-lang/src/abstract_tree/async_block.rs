use std::sync::Mutex;

use rayon::prelude::*;
use serde::{Deserialize, Serialize};

use crate::{
    context::Context,
    error::{RuntimeError, ValidationError},
};

use super::{Evaluation, ExpectedType, Run, Statement, Type, Validate};

#[derive(Clone, Debug, Eq, PartialEq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct AsyncBlock {
    statements: Vec<Statement>,
}

impl AsyncBlock {
    pub fn new(statements: Vec<Statement>) -> Self {
        Self { statements }
    }
}

impl Validate for AsyncBlock {
    fn validate(&self, _context: &mut Context, manage_memory: bool) -> Result<(), ValidationError> {
        for statement in &self.statements {
            statement.validate(_context, manage_memory)?;
        }

        Ok(())
    }
}

impl Run for AsyncBlock {
    fn run(
        self,
        _context: &mut Context,
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
}

impl ExpectedType for AsyncBlock {
    fn expected_type(&self, _context: &mut Context) -> Result<Type, ValidationError> {
        self.statements.first().unwrap().expected_type(_context)
    }
}
