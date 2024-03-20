use rayon::prelude::*;

use crate::{
    context::Context,
    error::{RuntimeError, ValidationError},
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

        self.statements
            .into_par_iter()
            .enumerate()
            .find_map_any(|(index, statement)| {
                let result = statement.node.run(_context);

                match result {
                    Ok(action) => {
                        if index == statement_count - 1 {
                            Some(Ok(action))
                        } else {
                            None
                        }
                    }
                    Err(runtime_error) => Some(Err(runtime_error)),
                }
            })
            .unwrap()
    }
}
