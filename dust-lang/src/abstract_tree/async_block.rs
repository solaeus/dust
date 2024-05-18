use std::sync::RwLock;

use rayon::prelude::*;

use crate::{
    context::Context,
    error::{RuntimeError, RwLockPoisonError, ValidationError},
};

use super::{AbstractNode, Action, Statement, Type};

#[derive(Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
pub struct AsyncBlock {
    statements: Vec<Statement>,
}

impl AsyncBlock {
    pub fn new(statements: Vec<Statement>) -> Self {
        Self { statements }
    }
}

impl AbstractNode for AsyncBlock {
    fn expected_type(&self, _context: &mut Context) -> Result<Type, ValidationError> {
        self.statements.last().unwrap().expected_type(_context)
    }

    fn validate(&self, _context: &mut Context, manage_memory: bool) -> Result<(), ValidationError> {
        for statement in &self.statements {
            statement.validate(_context, manage_memory)?;
        }

        Ok(())
    }

    fn run(self, _context: &mut Context, _manage_memory: bool) -> Result<Action, RuntimeError> {
        let statement_count = self.statements.len();
        let final_result = RwLock::new(Ok(Action::None));

        self.statements
            .into_par_iter()
            .enumerate()
            .find_map_first(|(index, statement)| {
                let result = statement.run(&mut _context.clone(), false);

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