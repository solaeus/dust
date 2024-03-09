use crate::{
    context::Context,
    error::{RuntimeError, ValidationError},
};

use super::{AbstractTree, Action, Statement, Type};

#[derive(Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
pub struct Block {
    statements: Vec<Statement>,
}

impl Block {
    pub fn new(statements: Vec<Statement>) -> Self {
        Self { statements }
    }
}

impl AbstractTree for Block {
    fn expected_type(&self, _context: &Context) -> Result<Type, ValidationError> {
        if let Some(statement) = self.statements.last() {
            statement.expected_type(_context)
        } else {
            Ok(Type::None)
        }
    }

    fn validate(&self, _context: &Context) -> Result<(), ValidationError> {
        for statement in &self.statements {
            statement.validate(_context)?;
        }

        Ok(())
    }

    fn run(self, _context: &Context) -> Result<Action, RuntimeError> {
        let mut previous = Action::None;

        for statement in self.statements {
            let action = statement.run(_context)?;
            previous = match action {
                Action::Return(value) => Action::Return(value),
                r#break => return Ok(r#break),
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
            Statement::Expression(Expression::Value(ValueNode::Integer(1))),
            Statement::Expression(Expression::Value(ValueNode::Integer(2))),
            Statement::Expression(Expression::Value(ValueNode::Integer(42))),
        ]);

        assert_eq!(
            block.run(&Context::new()),
            Ok(Action::Return(Value::integer(42)))
        )
    }

    #[test]
    fn expected_type_returns_type_of_final_statement() {
        let block = Block::new(vec![
            Statement::Expression(Expression::Value(ValueNode::String("42".to_string()))),
            Statement::Expression(Expression::Value(ValueNode::Integer(42))),
        ]);

        assert_eq!(block.expected_type(&Context::new()), Ok(Type::Integer))
    }
}
