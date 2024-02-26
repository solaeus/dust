use crate::{context::Context, error::RuntimeError, Value};

use super::{AbstractTree, Expression};

#[derive(Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
pub enum Logic<'src> {
    Equal(Expression<'src>, Expression<'src>),
    NotEqual(Expression<'src>, Expression<'src>),
    Greater(Expression<'src>, Expression<'src>),
    Less(Expression<'src>, Expression<'src>),
    GreaterOrEqual(Expression<'src>, Expression<'src>),
    LessOrEqual(Expression<'src>, Expression<'src>),
    And(Expression<'src>, Expression<'src>),
    Or(Expression<'src>, Expression<'src>),
    Not(Expression<'src>),
}

impl<'src> AbstractTree for Logic<'src> {
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
    use crate::abstract_tree::{Expression, ValueNode};

    use super::*;

    #[test]
    fn equal() {
        assert!(Logic::Equal(
            Expression::Value(ValueNode::Integer(42)),
            Expression::Value(ValueNode::Integer(42)),
        )
        .run(&Context::new())
        .unwrap()
        .as_boolean()
        .unwrap())
    }
}
