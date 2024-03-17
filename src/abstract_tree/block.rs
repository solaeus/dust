use crate::{
    context::Context,
    error::{RuntimeError, ValidationError},
};

use super::{AbstractTree, Action, Positioned, Statement, Type};

#[derive(Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
pub struct Block {
    statements: Vec<Positioned<Statement>>,
}

impl Block {
    pub fn new(statements: Vec<Positioned<Statement>>) -> Self {
        Self { statements }
    }
}

impl AbstractTree for Block {
    fn expected_type(&self, _context: &Context) -> Result<Type, ValidationError> {
        if let Some(statement) = self.statements.last() {
            statement.node.expected_type(_context)
        } else {
            Ok(Type::None)
        }
    }

    fn validate(&self, _context: &Context) -> Result<(), ValidationError> {
        for statement in &self.statements {
            statement.node.validate(_context)?;
        }

        Ok(())
    }

    fn run(self, _context: &Context) -> Result<Action, RuntimeError> {
        let mut previous = Action::None;

        for statement in self.statements {
            let action = statement.node.run(_context)?;
            previous = match action {
                Action::Return(value) => Action::Return(value),
                Action::None => Action::None,
                Action::Break => return Ok(action),
            };
        }

        Ok(previous)
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        abstract_tree::{Expression, ValueNode},
        Value,
    };

    use super::*;

    #[test]
    fn run_returns_value_of_final_statement() {
        let block = Block::new(vec![
            Statement::Expression(
                Expression::Value(ValueNode::Integer(1)).positioned((0..0).into()),
            )
            .positioned((0..0).into()),
            Statement::Expression(
                Expression::Value(ValueNode::Integer(2)).positioned((0..0).into()),
            )
            .positioned((0..0).into()),
            Statement::Expression(
                Expression::Value(ValueNode::Integer(42)).positioned((0..0).into()),
            )
            .positioned((0..0).into()),
        ]);

        assert_eq!(
            block.run(&Context::new()),
            Ok(Action::Return(Value::integer(42)))
        )
    }

    #[test]
    fn expected_type_returns_type_of_final_statement() {
        let block = Block::new(vec![
            Statement::Expression(
                Expression::Value(ValueNode::String("42".to_string())).positioned((0..0).into()),
            )
            .positioned((0..0).into()),
            Statement::Expression(
                Expression::Value(ValueNode::Integer(42)).positioned((0..0).into()),
            )
            .positioned((0..0).into()),
        ]);

        assert_eq!(block.expected_type(&Context::new()), Ok(Type::Integer))
    }
}
