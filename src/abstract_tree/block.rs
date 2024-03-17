use crate::{
    context::Context,
    error::{RuntimeError, ValidationError},
};

use super::{AbstractTree, Action, Statement, Type, WithPosition};

#[derive(Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
pub struct Block {
    statements: Vec<WithPosition<Statement>>,
}

impl Block {
    pub fn new(statements: Vec<WithPosition<Statement>>) -> Self {
        Self { statements }
    }

    pub fn last_statement(&self) -> &WithPosition<Statement> {
        self.statements.last().unwrap()
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
            previous = statement.node.run(_context)?;
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
            Statement::Expression(Expression::Value(ValueNode::Integer(1))).with_position((0, 0)),
            Statement::Expression(Expression::Value(ValueNode::Integer(2))).with_position((0, 0)),
            Statement::Expression(Expression::Value(ValueNode::Integer(42))).with_position((0, 0)),
        ]);

        assert_eq!(
            block.run(&Context::new()),
            Ok(Action::Return(Value::integer(42)))
        )
    }

    #[test]
    fn expected_type_returns_type_of_final_statement() {
        let block = Block::new(vec![
            Statement::Expression(Expression::Value(ValueNode::String("42".to_string())))
                .with_position((0, 0)),
            Statement::Expression(Expression::Value(ValueNode::Integer(42))).with_position((0, 0)),
        ]);

        assert_eq!(block.expected_type(&Context::new()), Ok(Type::Integer))
    }
}
