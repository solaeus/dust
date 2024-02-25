use crate::{context::Context, error::RuntimeError};

use super::{AbstractTree, Statement, Value};

#[derive(Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
pub enum Logic {
    Equal(Statement, Statement),
    NotEqual(Statement, Statement),
    Greater(Statement, Statement),
    Less(Statement, Statement),
    GreaterOrEqual(Statement, Statement),
    LessOrEqual(Statement, Statement),
    And(Statement, Statement),
    Or(Statement, Statement),
    Not(Statement),
}

impl AbstractTree for Logic {
    fn run(self, _context: &Context) -> Result<Value, RuntimeError> {
        let boolean = match self {
            Logic::Equal(left, right) => left.run(_context)? == right.run(_context)?,
            Logic::NotEqual(left, right) => left.run(_context)? != right.run(_context)?,
            Logic::Greater(left, right) => left.run(_context)? > right.run(_context)?,
            Logic::Less(left, right) => left.run(_context)? < right.run(_context)?,
            Logic::GreaterOrEqual(left, right) => left.run(_context)? >= right.run(_context)?,
            Logic::LessOrEqual(left, right) => left.run(_context)? <= right.run(_context)?,
            Logic::And(left, right) => {
                left.run(_context)?.as_boolean()? && right.run(_context)?.as_boolean()?
            }
            Logic::Or(left, right) => {
                left.run(_context)?.as_boolean()? || right.run(_context)?.as_boolean()?
            }
            Logic::Not(statement) => !statement.run(_context)?.as_boolean()?,
        };

        Ok(Value::boolean(boolean))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn equal() {
        assert!(Logic::Equal(
            Statement::Value(Value::integer(42)),
            Statement::Value(Value::integer(42)),
        )
        .run(&Context::new())
        .unwrap()
        .as_boolean()
        .unwrap())
    }
}
