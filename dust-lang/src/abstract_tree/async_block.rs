use std::sync::RwLock;

use rayon::prelude::*;

use crate::{
    context::Context,
    error::{RuntimeError, RwLockPoisonError, ValidationError},
};

use super::{AbstractNode, Action, Statement, Type, WithPosition};

#[derive(Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
pub struct AsyncBlock {
    statements: Vec<WithPosition<Statement>>,
}

impl AsyncBlock {
    pub fn new(statements: Vec<WithPosition<Statement>>) -> Self {
        Self { statements }
    }
}

impl AbstractNode for AsyncBlock {
    fn expected_type(&self, _context: &Context) -> Result<Type, ValidationError> {
        self.statements.last().unwrap().node.expected_type(_context)
    }

    fn validate(&self, _context: &Context) -> Result<(), ValidationError> {
        for statement in &self.statements {
            statement.node.validate(_context)?;
        }

        Ok(())
    }

    fn run(self, _context: &Context) -> Result<Action, RuntimeError> {
        let statement_count = self.statements.len();
        let final_result = RwLock::new(Ok(Action::None));

        self.statements
            .into_par_iter()
            .enumerate()
            .find_map_first(|(index, statement)| {
                let result = statement.node.run(_context);

                if index == statement_count - 1 {
                    let get_write_lock = final_result.write();

                    match get_write_lock {
                        Ok(mut final_result) => {
                            *final_result = result;
                            None
                        }
                        Err(_error) => Some(Err(RuntimeError::RwLockPoison(RwLockPoisonError))),
                    }
                } else {
                    None
                }
            })
            .unwrap_or(
                final_result
                    .into_inner()
                    .map_err(|_| RuntimeError::RwLockPoison(RwLockPoisonError))?,
            )
    }
}
